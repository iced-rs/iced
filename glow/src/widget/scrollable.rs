//! Navigate an endless amount of content with a scrollbar.
use crate::Renderer;

pub use iced_graphics::scrollable::{Scrollbar, Scroller, StyleSheet};
pub use iced_native::widget::scrollable::State;

/// A widget that can vertically display an infinite amount of content
/// with a scrollbar.
///
/// This is an alias of an `iced_native` scrollable with a default
/// `Renderer`.
pub type Scrollable<'a, Message> =
    iced_native::widget::Scrollable<'a, Message, Renderer>;
