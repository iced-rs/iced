//! Decorate content and apply alignment.
use crate::container;
use crate::{Backend, Renderer};

pub use iced_style::container::{Style, StyleSheet};

/// An element decorating some content.
///
/// This is an alias of an `iced_native` container with a default
/// `Renderer`.
pub type Container<'a, Message, Backend> =
    iced_native::Container<'a, Message, Renderer<Backend>>;

impl<B> iced_native::container::Renderer for Renderer<B>
where
    B: Backend,
{
    type Style = Box<dyn container::StyleSheet>;
}
