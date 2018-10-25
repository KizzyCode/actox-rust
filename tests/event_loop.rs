extern crate actox;

use ::actox::{ BlockingEventSource, PollingEventSource, EventHandler, EventLoop };
use ::std::{
	thread, cell::RefCell, collections::hash_map::DefaultHasher, hash::{ Hasher, Hash },
	time::{ Duration, UNIX_EPOCH },
};


pub struct Prng64;
impl Prng64 {
	pub fn next() -> u64 {
		thread_local!(static RAND_STATE: RefCell<DefaultHasher> = RefCell::new(DefaultHasher::new()));
		RAND_STATE.with(|state: &RefCell<DefaultHasher>| {
			// Capture state
			let mut state = state.borrow_mut();
			
			// Update state and return hash
			thread::current().id().hash(&mut *state);
			Self::time_now_ns().hash(&mut *state);
			state.finish()
		})
	}
	
	fn time_now_ns() -> u64 {
		let now = UNIX_EPOCH.elapsed().unwrap();
		(now.as_secs() * 1_000_000_000) + (now.subsec_nanos() as u64)
	}
}


struct TestEventSource {
	counter: u128,
	limit: u128
}
impl TestEventSource {
	pub fn new(limit: u128) -> Self {
		Self{ counter: 0, limit }
	}
}
impl BlockingEventSource<u128, &'static str> for TestEventSource {
	fn wait(&mut self) -> Option<Result<u128, &'static str>> {
		// Check for limit
		if self.counter == self.limit { return Some(Err("Limit reached")) }
		
		// Sleep from 0 to 128 ms and determine if we should returns sth. or not
		thread::sleep(Duration::from_millis(Prng64::next() % 128));
		if Prng64::next() % 2 == 0 { return None }
		
		// Get event and increment counter
		let result = Ok(self.counter);
		self.counter += 1;
		Some(result)
	}
}
impl PollingEventSource<u128, &'static str> for TestEventSource {
	fn poll(&mut self) -> Option<Result<u128, &'static str>> {
		// Check for limit
		if self.counter == self.limit {
			return Some(Err("Limit reached"))
		}
		
		// Determine if we should returns sth. or not
		if Prng64::next() % 2 == 0 { return None }
		
		// Get event and increment counter
		let result = Ok(self.counter);
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
		.map(|i| TestEventSource::new(i))
		.collect();
	
	// Create event handler
	struct TestEventHandler(u128);
	impl EventHandler<u128, &'static str> for TestEventHandler {
		fn handle_event(&mut self, event: Result<u128, &'static str>) -> Result<(), &'static str> {
			Ok(match event {
				Ok(event) => self.0 += event,
				Err(event) => eprintln!("Source failed with error: {}", event)
			})
		}
	}
	let event_handler = TestEventHandler(0);
	
	// Create and populate loop
	let mut event_loop = EventLoop::new();
	for source in sources {
		if Prng64::next() % 2 == 0 { event_loop.add_blocking_source(source); }
			else { event_loop.add_polling_source(source); }
	}
	
	// Run event loop and check result
	let handler = event_loop.start_sync(event_handler).unwrap();
	assert_eq!(handler.0, sum);
}