use ::{ ActoxResult, ActoxError, ActorPool };
use ::etrace::Error;
use ::std::{
	thread, collections::VecDeque, time::Duration,
	sync::mpsc::{ self, SyncSender, Receiver, TryRecvError }
};


/// An event returned by an event source
pub struct Event<E: Send + 'static> {
	/// The payload returned by the event source
	pub payload: E,
	/// The name/identifier of the source
	pub source: Box<AsRef<str> + Send>
}


/// A blocking event source
pub trait BlockingEventSource<E: Send + 'static, R: Send + 'static> where Self: Send + 'static {
	/// Waits for the next event
	///
	/// _Note: This function may block forever until either an event or an error occurres. However
	/// it is usually better to just block a reasonable time and return `Event::RetryLater` if no
	/// event occurred._
	///
	/// Returns either _`Some(event_result)`_ if an event/error occurred or _`None`_ if nothing
	/// happened
	fn wait(&mut self) -> Option<Result<Event<E>, Event<R>>>;
}
/// A polling event source
pub trait PollingEventSource<E: Send + 'static, R: Send + 'static> where Self: Send + 'static {
	/// Checks if an event is available
	///
	/// __Warning: Since this function is not executed in it's own private thread, no further events
	/// can be polled until the function returns.__
	///
	/// Returns either _`Some(event_result)`_ if an event/error occurred or _`None`_ if nothing
	/// happened
	fn poll(&mut self) -> Option<Result<Event<E>, Event<R>>>;
}


/// An `EventHandler`
pub trait EventHandler<E: Send + 'static, R: Send + 'static> {
	/// Handles the next `Event`
	///
	/// __Warning: if the function returns an error, the event-loop will terminate and also return
	/// this error__
	///
	/// Returns either _nothing_ or a __fatal__ error
	fn handle_event(&mut self, event: Result<Event<E>, Event<R>>) -> Result<(), R>;
}


/// An `EventLoop` that aggregates multiple `EventSource`s and calls an `EventHandler` if an event
/// occurres
pub struct EventLoop<E: Send + 'static, R: Send + 'static> {
	blocking: Vec<Box<BlockingEventSource<E, R>>>,
	polling: VecDeque<Box<PollingEventSource<E, R>>>
}
impl<E: Send + 'static, R: Send + 'static> EventLoop<E, R> {
	/// Creates a new `EventLoop`-instance
	pub fn new() -> Self {
		Self{ blocking: Vec::new(), polling: VecDeque::new() }
	}
	
	/// Adds the blocking `source` to the `EventLoop`
	///
	/// Parameter `source`: The `BlockingEventSource` to add
	pub fn add_blocking_source(&mut self, source: impl BlockingEventSource<E, R>) -> &mut Self {
		self.blocking.push(Box::new(source));
		self
	}
	/// Adds the polling `source` to the `EventLoop`
	///
	/// Parameter `source`: The `BlockingEventSource` to add
	pub fn add_polling_source(&mut self, source: impl PollingEventSource<E, R>) -> &mut Self {
		self.polling.push_back(Box::new(source));
		self
	}
	
	/// Starts the `EventLoop`
	///
	/// __Warning: This function will block until either `handler` returns an error or there is no
	/// `EventSource` alive anymore__
	///
	/// Parameter `handler`: The event handler to handle occurring events
	///
	/// Returns either _the event handler_ if there is no  `EventSource` alive anymore or rethrows
	/// the error returned by `handler`
	pub fn start_sync<H: EventHandler<E, R>>(self, mut handler: H) -> Result<H, R> {
		// Prepare aggregator
		let (sender, aggregator) =
			mpsc::sync_channel(self.blocking.len() + self.polling.len());
		
		// Aggregate blocking sources async
		for source in self.blocking {
			Self::aggregate_blocking_source_async(source, sender.clone())
		}
		
		// Aggregate polling sources
		Self::aggregate_polling_sources_async(self.polling, sender);
		
		// Start event loop
		'event_loop: loop {
			// Receive event from aggregator and handle event
			let event: Option<Result<Event<E>, Event<R>>> = ok_or!(aggregator.recv(), return Ok(handler));
			if let Some(e) = event { handler.handle_event(e)?; }
		}
	}
	
	/// Starts the `EventLoop` as an actor
	///
	/// The difference between this function and `start_sync` is that an actor runs in a background
	/// thread and has a globally accessible input-channel (referenced by a `name`) to send events
	/// to the event loop (the input-channel is just another for the `EventLoop`).
	///
	/// Parameters:
	///  - `name`: The name to reference the actor's input channel
	///  - `handler`: The event handler to handle occurring events
	///
	/// Returns either _nothing_ if the actor was registered successfully or
	/// `ActoxError::ExistsAlready` if there is already an actor registered under `name`
	pub fn start_actor(mut self, name: impl ToString, handler: impl EventHandler<E, R> + Send + 'static) -> ActoxResult<()>
		where R: From<Error<ActoxError>>
	{
		// Create channels and register
		let (sender, receiver) = mpsc::channel();
		try_err!(ActorPool::register(sender, name.to_string()));
		
		// Create event source for `receiver`
		self.add_polling_source(MpscEventSource {
			input: receiver,
			name: format!("\"{}\"::ActorInput", name.to_string())
		});
		
		// Start runloop async
		let name = name.to_string();
		thread::spawn(move || {
			let _ = self.start_sync(handler).map(|_| ());
			let _ = ActorPool::unregister(name.to_string());
		});
		Ok(())
	}
	
	
	/// Starts a background thread to wait for and aggregate events of the blocking `source`
	fn aggregate_blocking_source_async(mut source: Box<BlockingEventSource<E, R>>, aggregator: SyncSender<Option<Result<Event<E>, Event<R>>>>) {
		thread::spawn(move || {
			let mut stop = false;
			while !stop {
				// Wait for next event set stop flag on error
				let event = source.wait();
				if let Some(Err(_)) = event { stop = true }
				
				// Send event
				ok_or!(aggregator.send(event), return);
			}
		});
	}
	/// Starts a background thread to wait for and aggregate events of the polling `sources`
	fn aggregate_polling_sources_async(mut sources: VecDeque<Box<PollingEventSource<E, R>>>, aggregator: SyncSender<Option<Result<Event<E>, Event<R>>>>) {
		thread::spawn(move || while !sources.is_empty() {
			// Capture if a source had an event
			let mut had_event = false;
			
			// Poll each source
			for _ in 0..sources.len() {
				// Get source and poll for event
				let mut source = sources.pop_front().unwrap();
				let event = source.poll();
				
				// Set `had_event` flag if necessary and push source back to queue
				if event.is_some() { had_event = true }
				match event {
					None | Some(Ok(_)) => sources.push_back(source),
					Some(Err(_)) => (),
				}
				
				// Send event to aggregator and push source back to queue
				ok_or!(aggregator.send(event), return);
			}
			
			// Sleep if no event occurred
			if !had_event { thread::sleep(Duration::from_millis(100)) }
		});
	}
}


/// An event source for a `mpsc::Receiver`
struct MpscEventSource<E: Send + 'static> {
	input: Receiver<E>,
	name: String
}
impl<E: Send + 'static, R: Send + 'static + From<Error<ActoxError>>> PollingEventSource<E, R> for MpscEventSource<E> {
	fn poll(&mut self) -> Option<Result<Event<E>, Event<R>>> {
		match self.input.try_recv() {
			Ok(event) => Some(Ok(Event {
				payload: event,
				source: Box::new(self.name.clone())
			})),
			Err(TryRecvError::Disconnected) => Some(Err(Event {
				payload: new_err!(ActoxError::EndpointNotConnected).into(),
				source: Box::new(self.name.clone())
			})),
			Err(TryRecvError::Empty) => None
		}
	}
}