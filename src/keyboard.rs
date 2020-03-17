//! Listen and react to keyboard events.
#[cfg(not(target_arch = "wasm32"))]
pub use iced_winit::input::keyboard::{KeyCode, ModifiersState};

#[cfg(target_arch = "wasm32")]
pub use iced_web::keyboard::{KeyCode, ModifiersState};
