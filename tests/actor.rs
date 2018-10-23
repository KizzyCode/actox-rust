extern crate actox;

use ::actox::{ ActoxError, ActorPool, Event, EventHandler, EventLoop };
use ::std::sync::mpsc::{ self, Sender };


enum Message {
	Put(u128),
	Get(Sender<u128>)
}


#[derive(Default)]
struct Collector {
	sum: u128
}
impl EventHandler<Message, ActoxError> for Collector {
	fn handle_event(&mut self, event: Result<Event<Message>, Event<ActoxError>>) -> Result<(), ActoxError> {
		let event = match event {
			Ok(event) => event,
			_ => unreachable!()
		};
		
		Ok(match event.payload {
			Message::Put(num) => self.sum += num,
			Message::Get(sender) => sender.send(self.sum).unwrap(),
		})
	}
}


#[test] fn test_ok() {
	// Start actor
	EventLoop::new().start_actor("TestOk::Actor", Collector::default()).unwrap();
	
	// Send numbers
	let range = 0..77_777;
	for i in range.clone() { ActorPool::send("TestOk::Actor", Message::Put(i)).unwrap() }
	
	// Request sum
	let (sender, receiver) = mpsc::channel();
	ActorPool::send("TestOk::Actor", Message::Get(sender)).unwrap();
	
	// Wait for sum and validate it
	let sum = receiver.recv().unwrap();
	assert_eq!(sum, range.sum());
}

#[test] fn test_err_notfound() {
	assert_eq!(
		ActorPool::send("TestNotFound::Actor", ()).unwrap_err().kind,
		ActoxError::NotFound
	)
}

#[test] fn test_err_existsalready() {
	// Register first actor
	EventLoop::new().start_actor("TestExistsAlready::Actor", Collector::default()).unwrap();
	
	// Try to register another "Collector"
	assert_eq!(
		EventLoop::new().start_actor("TestExistsAlready::Actor", Collector::default()).unwrap_err().kind,
		ActoxError::ExistsAlready
	)
}

#[test] fn test_err_typemismatch() {
	// Start actor
	EventLoop::new().start_actor("TestTypeMismatch::Actor", Collector::default()).unwrap();
	
	// Send invalid typed message
	assert_eq!(
		ActorPool::send("TestTypeMismatch::Actor", ()).unwrap_err().kind,
		ActoxError::TypeMismatch
	)
}