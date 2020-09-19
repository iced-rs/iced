//! Show toggle controls using togglers.
use crate::Renderer;

pub use iced_graphics::toggler::{Style, StyleSheet};

/// A toggler that can be toggled.
///
/// This is an alias of an `iced_native` checkbox with an `iced_wgpu::Renderer`.
pub type Toggler<Message> = iced_native::Toggler<Message, Renderer>;
