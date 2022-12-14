//! Access the clipboard.
#[cfg(all(not(target_arch = "wasm32"), not(feature = "wayland")))] // TODO support in wayland
pub use crate::runtime::clipboard::{read, write};
