//! Display fields that can be filled with text.
//!
//! A [`TextInput`] has some local [`State`].
pub(crate) mod editor;
pub(crate) mod value;

pub mod cursor;

#[cfg(feature = "wayland")]
mod text_input_wayland;
#[cfg(feature = "wayland")]
pub use text_input_wayland::*;
#[cfg(not(feature = "wayland"))]
mod text_input;
#[cfg(not(feature = "wayland"))]
pub use text_input::*;


