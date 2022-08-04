#![doc = include_str!("../README.md")]
#![deny(unsafe_code)]

mod bus;
mod queue;
mod subscriber;

pub use crate::{bus::Bus, subscriber::Subscriber};
