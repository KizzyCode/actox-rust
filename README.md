[![License](https://img.shields.io/badge/License-BSD--2--Clause-blue.svg)](https://opensource.org/licenses/BSD-2-Clause)
[![License](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)
[![Travis CI](https://travis-ci.org/KizzyCode/actox.svg?branch=master)](https://travis-ci.org/KizzyCode/actox)
[![AppVeyor CI](https://ci.appveyor.com/api/projects/status/github/KizzyCode/actox?svg=true)](https://ci.appveyor.com/project/KizzyCode/actox)

# actox
Welcome to my `actox`-library 🎉


## What this crate is:
This crate is small and nearly dependency-less crate that implements a simple aggregating event-loop and a cheap actor
implementation with a global actor-pool.

The event-loop supports two types of event sources: blocking ones and polling ones. Blocking ones are especially useful
if you only have a non-blocking API available (e.g. DNS-resolution) - the downside of them is that each blocking source
needs to be queried in it's own thread. On the opposite the polling sources do not block and are queried in a single
separate thread (thus requiring much less overhead).

Each event-loop can be started either synchronously or asynchronously as a named, globally available actor.


## Why should I use this crate?
You probably shouldn't. There are other solutions like [actix](https://crates.io/crates/actix) out there that are much
more complete, much better tested and probably much more efficient.

But if you want to avoid the [dependeny hell](https://en.wikipedia.org/wiki/Dependency_hell) or if you want to
understand the crates your'e using, this crate _could_ suite you 😇


# Dependencies
Only my [`etrace`-crate](https://crates.io/crates/etrace) for traceable error handling.

# Build Documentation and Library:
To build and open the documentation, go into the project's root-directory and run `cargo doc --release --open`

To build this library, change into the projects root-directory and run `cargo build --release`; you can find the build
in `target/release`.