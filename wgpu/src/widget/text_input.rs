//! Display fields that can be filled with text.
//!
//! A [`TextInput`] has some local [`State`].
use crate::Renderer;

pub use iced_graphics::text_input::{Style, StyleSheet};
pub use iced_native::text_input::State;

/// A field that can be filled with text.
///
/// This is an alias of an `iced_native` text input with an `iced_wgpu::Renderer`.
pub type TextInput<Message> = iced_native::TextInput<Message, Renderer>;
