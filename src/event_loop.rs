use ::{ Result, ActorError, ActorPool };
use ::std::{ thread, sync::mpsc::{ self, Receiver } };


/// An `EventSource`
pub trait EventSource<T: Send + 'static> where Self: Send + 'static {
	/// Waits for the next event
	///
	/// _Note: This function __must__ block until either an event or an error occurres_
	///
	/// __Warning: If the `EventSource` returns an error, it will be removed from the `EventLoop`__
	///
	/// Returns either _the next event_ or a corresponding __fatal__ error
	fn next_event(&mut self) -> Result<T>;
}
/// An `EventHandler`
pub trait EventHandler<T: Send + 'static> {
	/// Handles the next `event`
	///
	/// __Warning: if the function returns an error, the `EventLoop` will terminate and rethrow the
	/// error__
	///
	/// Returns either _nothing_ or a corresponding __fatal__ error
	fn handle_event(&mut self, event: T) -> Result<()>;
}


/// An `EventLoop` that aggregates multiple `EventSource`s and calls an `EventHandler` if an event
/// occurres
pub struct EventLoop<T: Send + 'static> {
	event_sources: Vec<Box<EventSource<T>>>
}
impl<T: Send> EventLoop<T> {
	/// Creates a new `EventLoop`-instance
	pub fn new() -> Self {
		Self{ event_sources: Vec::new() }
	}
	
	/// Adds `source` to the `EventLoop`
	///
	/// Parameter `source`: The `EventSource` to add
	pub fn add_event_source(&mut self, source: impl EventSource<T>) -> &mut Self {
		self.event_sources.push(Box::new(source));
		self
	}
	
	/// Starts the `EventLoop`
	///
	/// __Warning: This function will block until either `handler` returns an error or there is no
	/// `EventSource` alive anymore__
	///
	/// Parameter `handler`: The event handler to handle occurring events
	///
	/// Returns either _nothing_ if there is no  `EventSource` alive anymore or rethrows the error
	/// returned by `handler`
	pub fn start_sync(self, mut handler: impl EventHandler<T>) -> Result<()> {
		// Start aggregator threads
		let (sender, aggregator) = mpsc::sync_channel(self.event_sources.len());
		for mut source in self.event_sources {
			let _sender = sender.clone();
			thread::spawn(move || loop {
				// Wait for next event
				let event: T = ok_or!(source.next_event(), e, {
					warn!("Event source terminated with error ", e);
					return;
				});
				// Send event to aggregator
				ok_or!(_sender.send(event), {
					warn!("Cannot send event to aggregator; stopping event source");
					return;
				});
			});
		}
		
		// Start loop
		loop {
			// Receive event from aggregator
			let event: T = ok_or!(aggregator.recv(), {
				warn!("No event sources available; stopping event loop");
				return Ok(())
			});
			// Handle event
			try_err!(handler.handle_event(event));
		}
	}
	
	/// Starts the `EventLoop` as an actor
	///
	/// The difference between this function and `start_sync` is that an actor runs in a background
	/// thread and has a globally accessible input-channel (referenced by a `name`) to send events
	/// to the event loop (the input-channel is just another `EventSource` for the `EventLoop`).
	///
	/// Parameters:
	///  - `name`: The name to reference the actor's input channel
	///  - `handler`: The event handler to handle occurring events
	///
	/// Returns either _nothing_ if the actor was registered successfully or
	/// `ActorError::ExistsAlready` if there is already an actor registered under `name`
	pub fn start_actor(mut self, name: impl ToString, handler: impl EventHandler<T> + Send + 'static) -> Result<()> {
		// Create channels and register
		let (sender, receiver) = mpsc::channel();
		try_err!(ActorPool::register(sender, name.to_string()));
		
		// Create event source for `receiver`
		struct MpscEventSource<T: Send + 'static>(Receiver<T>);
		impl<T: Send + 'static> EventSource<T> for MpscEventSource<T> {
			fn next_event(&mut self) -> Result<T> {
				Ok(ok_or!(self.0.recv(), throw_err!(ActorError::EndpointNotConnected)))
			}
		}
		self.add_event_source(MpscEventSource(receiver));
		
		// Start runloop async
		let name = name.to_string();
		thread::spawn(move || {
			ok_or!(self.start_sync(handler), e, warn!("Actor crashed with error ", e));
			ok_or!(
				ActorPool::unregister(name.to_string()),
				warn!("Failed to unregister actor ", name.to_string())
			)
		});
		Ok(())
	}
}