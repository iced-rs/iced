//! Configure the window of your application in native platforms.
pub use iced_native::window::Icon;
pub use iced_native::window::Position;
pub use iced_native::window::Settings;

#[cfg(not(target_arch = "wasm32"))]
pub use crate::runtime::window::*;
