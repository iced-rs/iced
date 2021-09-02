//! Access the clipboard.
#[cfg(not(target_arch = "wasm32"))]
pub use crate::runtime::clipboard::{read, write};
