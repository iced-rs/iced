//! Listen and react to keyboard events.
#[cfg(feature = "iced_sctk")]
pub use iced_sctk::input::keyboard;

#[cfg(feature = "iced_winit")]
pub use iced_winit::input::keyboard;

#[cfg(target_arch = "wasm32")]
pub use iced_web::keyboard;

pub use keyboard::{KeyCode, ModifiersState};
