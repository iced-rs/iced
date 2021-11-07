//! Decorate content and apply alignment.
use crate::Renderer;

/// An element decorating some content.
///
/// This is an alias of an `iced_native` tooltip with a default
/// `Renderer`.
pub type Tooltip<'a, Message, Backend> =
    iced_native::widget::Tooltip<'a, Message, Renderer<Backend>>;

pub use iced_native::widget::tooltip::Position;
