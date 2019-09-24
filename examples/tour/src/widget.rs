#[cfg(target_arch = "wasm32")]
pub use iced_web::*;

#[cfg(not(target_arch = "wasm32"))]
pub use crate::iced_ggez::*;
