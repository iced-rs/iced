//! Create choices using tab buttons.
//!
//! [`Tab`]: type.Tab.html
use crate::Renderer;

pub use iced_graphics::tab::{
    Indicator, Position, Style, StyleDefaultVertical, StyleSheet,
};

/// Create choices using tab buttons.
///
/// This is an alias of an `iced_native` tab with an `iced_glow::Renderer`.
pub type Tab<'a, Message> = iced_native::Tab<'a, Message, Renderer>;
