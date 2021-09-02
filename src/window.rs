//! Configure the window of your application in native platforms.
mod mode;
mod position;
mod settings;

pub mod icon;

pub use icon::Icon;
pub use mode::Mode;
pub use position::Position;
pub use settings::Settings;

#[cfg(not(target_arch = "wasm32"))]
pub use crate::runtime::window::{move_to, resize};
