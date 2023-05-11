//! Configure the window of your application in native platforms.
mod position;
mod settings;

pub mod icon;

pub use icon::Icon;
pub use position::Position;
pub use settings::{PlatformSpecific, Settings};

pub use crate::core::window::*;
pub use crate::runtime::window::*;
