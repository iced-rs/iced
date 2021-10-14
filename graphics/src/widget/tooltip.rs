//! Decorate content and apply alignment.
use crate::backend::{self, Backend};
use crate::Renderer;

/// An element decorating some content.
///
/// This is an alias of an `iced_native` tooltip with a default
/// `Renderer`.
pub type Tooltip<'a, Message, Backend> =
    iced_native::Tooltip<'a, Message, Renderer<Backend>>;

pub use iced_native::tooltip::Position;

impl<B> iced_native::tooltip::Renderer for Renderer<B>
where
    B: Backend + backend::Text,
{
    const DEFAULT_PADDING: u16 = 5;
}
