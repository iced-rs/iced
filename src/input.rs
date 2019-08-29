//! Map your system events into input events that the runtime can understand.
pub mod keyboard;
pub mod mouse;

mod button_state;

pub use button_state::ButtonState;
