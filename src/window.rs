//! Configure the window of your application in native platforms.

#[cfg(feature = "winit")]
pub mod icon;
#[cfg(feature = "winit")]
mod position;
#[cfg(feature = "winit")]
mod settings;

#[cfg(feature = "winit")]
pub use icon::Icon;
#[cfg(feature = "winit")]
pub use position::Position;

#[cfg(feature = "winit")]
pub use settings::{PlatformSpecific, Settings};

pub use crate::core::window::*;
pub use crate::runtime::window::*;
