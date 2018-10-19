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

/// Log a warning (with line info etc.)
///
/// Parameter `$($warning:expr),+`: A variadic list of items that implement `::std::fmt::Display`
#[macro_export] macro_rules! warn {
	($($warning:expr),+) => ({
		// Concat warning
		use ::std::fmt::Write;
		let mut warning = ::std::string::String::new();
		$(write!(&mut warning, "{}", $warning).unwrap();)+
		
		// Create and print error
		eprintln!("{}", new_err!($crate::ActorError::Warning, warning));
	});
}

mod event_loop;
mod actor_pool;


pub use ::event_loop::{ EventSource, EventHandler, EventLoop };
pub use ::actor_pool::ActorPool;


/// An actor related error
#[derive(Debug, Clone)]
pub enum ActorError {
	/// A warning (usually used for logging purposes)
	Warning,
	/// An element was not found
	NotFound,
	/// An element (with the same name/ID) exists already
	ExistsAlready,
	/// The endpoint is not connected (anymore)
	EndpointNotConnected,
	/// Invalid message type
	TypeMismatch,
	/// Another `etrace` error
	OtherError(::etrace::WrappedError)
}
/// Syntactic sugar for `::std::result::Result<T, ::etrace::Error<ActorError>>`
pub type Result<T> = ::std::result::Result<T, ::etrace::Error<ActorError>>;