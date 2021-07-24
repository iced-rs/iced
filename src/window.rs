//! Configure the window of your application in native platforms.
mod mode;
mod position;
mod settings;

pub mod icon;

pub use icon::Icon;
pub use mode::Mode;
pub use position::Position;
pub use settings::Settings;
