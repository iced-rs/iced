//! Create choices using radio buttons.
use crate::Renderer;

pub use iced_graphics::radio::{Style, StyleSheet};

/// A circular button representing a choice.
///
/// This is an alias of an `iced_native` radio button with an
/// `iced_wgpu::Renderer`.
pub type Radio<'a, Message> = iced_native::widget::Radio<'a, Message, Renderer>;
