use ::{ ActorError, Result };
use ::std::{
	ptr, collections::HashMap, any::Any,
	sync::{ mpsc::Sender, Mutex, Arc, atomic::{ AtomicPtr, Ordering } }
};


/// A global `ActorPool` to register input-channels under a name
pub struct ActorPool;
impl ActorPool {
	/// Registers a new `actor_input` under `name`
	///
	/// Parameters:
	///  - `actor_input`: The input-channel to register
	///  - `name`: The name under which the input-channel should be registered
	///
	/// Returns either _nothing_ or an `ActorError::ExistsAlready` if an actor with the same `name`
	/// is registered
	pub fn register<T: Send + 'static>(actor_input: Sender<T>, name: impl ToString) -> Result<()> {
		// Get pool instance
		let pool = Self::pool();
		let mut pool = pool.lock().unwrap();
		
		// Insert sender
		let name = name.to_string();
		if !pool.contains_key(&name) { pool.insert(name, Box::new(actor_input)); }
			else { throw_err!(ActorError::ExistsAlready) }
		Ok(())
	}
	/// Unregisters an input-channel
	///
	/// Parameter `name`: The name under which the input-channel to unregister was registered
	///
	/// Returns either _nothing_ or an `ActorError::NotFound` if no input-channel with `name` was
	/// registered
	pub fn unregister(name: impl ToString) -> Result<()> {
		// Get pool instance
		let pool = Self::pool();
		let mut pool = pool.lock().unwrap();
		
		// Get and cast actor
		some_or!(pool.remove(&name.to_string()), throw_err!(ActorError::NotFound));
		Ok(())
	}
	
	/// Sends a `message` to an input-channel registered under `name`
	///
	/// Parameters:
	///  - `name`: The name under which the input-channel was registered
	///  - `message`: The message to send to the input-channel
	///
	/// Returns either _nothing_ or a corresponding `ActorError`
	pub fn send<T: Send + 'static>(name: impl ToString, message: T) -> Result<()> {
		// Get pool instance
		let pool = Self::pool();
		let pool = pool.lock().unwrap();
		
		// Get and cast actor
		let actor: &Box<Any> = some_or!(pool.get(&name.to_string()), throw_err!(ActorError::NotFound));
		let actor: &Sender<T> = some_or!(actor.as_ref().downcast_ref(), throw_err!(ActorError::TypeMismatch));
		
		// Send message
		Ok(ok_or!(actor.send(message), throw_err!(ActorError::EndpointNotConnected)))
	}
	
	fn pool() -> Arc<Mutex<HashMap<String, Box<Any>>>> {
		static POOL: AtomicPtr<Arc<Mutex<HashMap<String, Box<Any>>>>> =
			AtomicPtr::new(ptr::null_mut());
		POOL.compare_and_swap(
			ptr::null_mut(),
			Box::into_raw(Box::new(Arc::new(Mutex::new(HashMap::new())))),
			Ordering::SeqCst
		);
		unsafe{ POOL.load(Ordering::SeqCst).as_ref().unwrap().clone() }
	}
}