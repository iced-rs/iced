#[doc(no_inline)]
pub use iced_native::*;
pub use winit;

pub mod conversion;

mod application;

pub use application::Application;

// We disable debug capabilities on release builds unless the `debug` feature
// is explicitly enabled.
#[cfg_attr(feature = "debug", path = "debug/basic.rs")]
#[cfg_attr(not(feature = "debug"), path = "debug/null.rs")]
mod debug;

use debug::Debug;
