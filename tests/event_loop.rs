extern crate actox;
extern crate rand;

use ::actox::{ Event, BlockingEventSource, PollingEventSource, EventHandler, EventLoop };
use ::rand::{ Rng, thread_rng };
use ::std::{ thread, time::Duration };


struct TestEventSource {
	name: String,
	counter: u128,
	limit: u128
}
impl TestEventSource {
	pub fn new(name: String, limit: u128) -> Self {
		Self{ name, counter: 0, limit }
	}
}
impl BlockingEventSource<u128, &'static str> for TestEventSource {
	fn wait(&mut self) -> Option<Result<Event<u128>, Event<&'static str>>> {
		// Check for limit
		if self.counter == self.limit {
			return Some(Err(Event{ payload: "Limit reached", source: Box::new(self.name.clone()) }))
		}
		
		// Sleep from 0 to 100 ms and determine if we should returns sth. or not
		thread::sleep(Duration::from_millis(thread_rng().gen_range(0, 100)));
		if thread_rng().gen() { return None }
		
		// Get event and increment counter
		let result = Ok(Event{ payload: self.counter, source: Box::new(self.name.clone()) });
		self.counter += 1;
		Some(result)
	}
}
impl PollingEventSource<u128, &'static str> for TestEventSource {
	fn poll(&mut self) -> Option<Result<Event<u128>, Event<&'static str>>> {
		// Check for limit
		if self.counter == self.limit {
			return Some(Err(Event{ payload: "Limit reached", source: Box::new(self.name.clone()) }))
		}
		
		// Determine if we should returns sth. or not
		if thread_rng().gen() { return None }
		
		// Get event and increment counter
		let result = Ok(Event{ payload: self.counter, source: Box::new(self.name.clone()) });
		self.counter += 1;
		Some(result)
	}
}


#[test] fn test() {
	// Define range
	let range = 0u128..77;
	let sum = range.clone()
		.map(|i| (0..i).sum::<u128>())
		.sum();
	
	// Create event sources
	let sources: Vec<TestEventSource> = range.clone()
		.map(|i| TestEventSource::new(format!("<TestEventSource>::{}", i), i))
		.collect();
	
	// Create event handler
	struct TestEventHandler(u128);
	impl EventHandler<u128, &'static str> for TestEventHandler {
		fn handle_event(&mut self, event: Result<Event<u128>, Event<&'static str>>) -> Result<(), &'static str> {
			Ok(match event {
				Ok(event) => self.0 += event.payload,
				Err(event) => eprintln!("Source \"{}\" failed with error: {}", event.source.as_ref().as_ref(), event.payload)
			})
		}
	}
	let event_handler = TestEventHandler(0);
	
	// Create and populate loop
	let mut event_loop = EventLoop::new();
	for source in sources {
		if thread_rng().gen() { event_loop.add_blocking_source(source); }
			else { event_loop.add_polling_source(source); }
	}
	
	// Run event loop and check result
	let handler = event_loop.start_sync(event_handler).unwrap();
	assert_eq!(handler.0, sum);
}