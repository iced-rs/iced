//! Map your system events into input events that the runtime can understand.
pub mod keyboard;
pub mod mouse;
pub mod touch;

mod button_state;

pub use button_state::ButtonState;
