//! Display fields that can only be filled with numeric type.
//!
//! A [`TextInput`] has some local [`State`].
use crate::Renderer;

pub use iced_graphics::number_input::{Style, StyleSheet};
pub use iced_native::number_input::State;

/// A field that can only be filled with numeric type.
///
/// This is an alias of an `iced_native` text input with an `iced_wgpu::Renderer`.
pub type NumberInput<'a, T, Message> = iced_native::NumberInput<'a, T, Message, Renderer>;