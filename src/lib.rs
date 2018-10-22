//! This crate is small and nearly dependency-less crate that implements a simple aggregating
//! event-loop and a cheap actor implementation with a global actor-pool.
//!
//! ## Why should I use this crate?
//! You probably shouldn't. There are other solutions like [actix](https://crates.io/crates/actix)
//! out there that are much mor complete, much better tested and probably much more efficient.
//!
//! But if you want to avoid the [dependeny hell](https://en.wikipedia.org/wiki/Dependency_hell) or
//! if you want to understand the crates your'e using, this crate _could_ suite you 😇

#[macro_use] pub extern crate etrace;

mod event_loop;
mod actor_pool;


pub use ::event_loop::{ Event, BlockingEventSource, PollingEventSource, EventHandler, EventLoop };
pub use ::actor_pool::ActorPool;


/// An actor related error
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum ActoxError {
	/// A warning (usually used for logging purposes)
	Warning,
	/// An element was not found
	NotFound,
	/// An element (with the same name/ID) exists already
	ExistsAlready,
	/// The endpoint is not connected (anymore)
	EndpointNotConnected,
	/// Invalid message type
	TypeMismatch
}
impl From<::etrace::Error<ActoxError>> for ActoxError {
	fn from(error: ::etrace::Error<ActoxError>) -> Self {
		return error.kind
	}
}
/// Syntactic sugar for `Result<T, ::etrace::Error<ActoxError>>`
pub type ActoxResult<T> = Result<T, ::etrace::Error<ActoxError>>;


