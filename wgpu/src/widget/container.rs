//! Decorate content and apply alignment.
use crate::Renderer;

pub use iced_graphics::container::{Style, StyleSheet};

/// An element decorating some content.
///
/// This is an alias of an `iced_native` container with a default
/// `Renderer`.
pub type Container<'a, Message> =
    iced_native::widget::Container<'a, Message, Renderer>;
