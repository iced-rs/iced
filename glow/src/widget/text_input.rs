//! Display fields that can be filled with text.
//!
//! A [`TextInput`] has some local [`State`].
//!
//! [`TextInput`]: struct.TextInput.html
//! [`State`]: struct.State.html
use crate::Renderer;

pub use iced_native::text_input::State;
pub use iced_style::text_input::{Style, StyleSheet};

/// A field that can be filled with text.
///
/// This is an alias of an `iced_native` text input with an `iced_wgpu::Renderer`.
pub type TextInput<'a, Message> = iced_native::TextInput<'a, Message, Renderer>;
