//! Navigate an endless amount of content with a scrollbar.
use crate::Renderer;

pub use iced_graphics::scrollable_hor::{Scrollbar, Scroller, StyleSheet};
pub use iced_native::scrollable_hor::State;

/// A widget that can vertically display an infinite amount of content
/// with a scrollbar.
///
/// This is an alias of an `iced_native` scrollable with a default
/// `Renderer`.
pub type ScrollableHor<'a, Message> =
    iced_native::ScrollableHor<'a, Message, Renderer>;
