//! Structures for handling screenshots

#[cfg(not(target_arch = "wasm32"))]
pub use crate::runtime::window::Screenshot;
