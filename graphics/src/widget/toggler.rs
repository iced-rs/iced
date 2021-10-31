//! Show toggle controls using togglers.
use crate::Renderer;

pub use iced_style::toggler::{Style, StyleSheet};

/// A toggler that can be toggled.
///
/// This is an alias of an `iced_native` toggler with an `iced_wgpu::Renderer`.
pub type Toggler<Message, Backend> =
    iced_native::widget::Toggler<Message, Renderer<Backend>>;
