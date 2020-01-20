//! Asynchronous tasks for GUI programming, inspired by Elm.
#![deny(missing_docs)]
#![deny(missing_debug_implementations)]
#![deny(unused_results)]
#![deny(unsafe_code)]
#![deny(rust_2018_idioms)]
pub use futures;

mod command;
mod runtime;

pub mod executor;
pub mod subscription;

pub use command::Command;
pub use executor::Executor;
pub use runtime::Runtime;
pub use subscription::Subscription;
