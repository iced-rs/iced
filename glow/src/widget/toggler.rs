//! Show toggle controls using togglers.
use crate::Renderer;

pub use iced_graphics::toggler::{Style, StyleSheet};

/// A toggler that can be toggled.
///
/// This is an alias of an `iced_native` checkbox with an `iced_wgpu::Renderer`.
pub type Toggler<'a, Message> =
    iced_native::widget::Toggler<'a, Message, Renderer>;
