[![License](https://img.shields.io/badge/License-BSD--2--Clause-blue.svg)](https://opensource.org/licenses/BSD-2-Clause)
[![License](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)
[![AppVeyor CI](https://ci.appveyor.com/api/projects/status/github/KizzyCode/actox-rust?svg=true)](https://ci.appveyor.com/project/KizzyCode/actox-rust)
[![docs.rs](https://docs.rs/actox/badge.svg)](https://docs.rs/actox)
[![crates.io](https://img.shields.io/crates/v/actox.svg)](https://crates.io/crates/actox)
[![Download numbers](https://img.shields.io/crates/d/actox.svg)](https://crates.io/crates/actox)
[![dependency status](https://deps.rs/crate/actox/0.2.1/status.svg)](https://deps.rs/crate/actox/0.2.1)

# actox
Welcome to my `actox`-library 🎉

`actox` is small and dependency-less crate that implements a shared message dispatcher to publish and subscribe on
multible topics from different threads. This allows for trivial multi-producer/multi-consumer networks without the need
to manually establish and manage dedicated communication channels between all participants.

## Design considerations
This crate is opinionated. The main goal is to have simple code that hopefully just works. No external dependencies, no
super-duper custom lock-free maps etc. No `unsafe`. Therefore is probably not super efficient or fast (even though it
achieves a throughput of about 7 million messages per second on my platform).
