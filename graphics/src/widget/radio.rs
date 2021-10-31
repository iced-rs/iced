//! Create choices using radio buttons.
use crate::Renderer;

pub use iced_style::radio::{Style, StyleSheet};

/// A circular button representing a choice.
///
/// This is an alias of an `iced_native` radio button with an
/// `iced_wgpu::Renderer`.
pub type Radio<'a, Message, Backend> =
    iced_native::widget::Radio<'a, Message, Renderer<Backend>>;
