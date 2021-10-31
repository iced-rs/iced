//! Display fields that can be filled with text.
//!
//! A [`TextInput`] has some local [`State`].
use crate::Renderer;

pub use iced_native::widget::text_input::State;
pub use iced_style::text_input::{Style, StyleSheet};

/// A field that can be filled with text.
///
/// This is an alias of an `iced_native` text input with an `iced_wgpu::Renderer`.
pub type TextInput<'a, Message, Backend> =
    iced_native::widget::TextInput<'a, Message, Renderer<Backend>>;
