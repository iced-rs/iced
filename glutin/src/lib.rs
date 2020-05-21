//#![deny(missing_docs)]
#![deny(missing_debug_implementations)]
#![deny(unused_results)]
#![deny(unsafe_code)]
#![forbid(rust_2018_idioms)]

#[doc(no_inline)]
pub use iced_native::*;

pub mod application;

pub use application::Application;

pub use iced_winit::settings::{self, Settings};
pub use iced_winit::Mode;
