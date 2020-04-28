//! Listen and react to mouse events.
#[cfg(not(target_arch = "wasm32"))]
pub use iced_winit::input::mouse::{Button, Event, ScrollDelta};
