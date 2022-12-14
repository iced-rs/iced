//! Configure the window of your application in native platforms.
mod position;
mod settings;
#[cfg(feature = "winit")]

pub mod icon;
#[cfg(feature = "winit")]
pub use icon::Icon;
pub use position::Position;
pub use settings::Settings;

#[cfg(not(target_arch = "wasm32"))]
pub use crate::runtime::window::resize;
#[cfg(not(any(target_arch = "wasm32", feature = "wayland")))]
pub use crate::runtime::window::move_to;

pub use iced_native::window::Id;
