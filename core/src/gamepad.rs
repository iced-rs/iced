//! Listen to gamepad events.

mod axis;
mod button;
mod event;
mod thumbstick;

pub use axis::Axis;
pub use button::Button;
pub use event::Event;
pub use thumbstick::Thumbstick;

pub use gilrs::GamepadId as Id;
