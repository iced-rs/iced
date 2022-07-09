//! Navigate an endless amount of content with a scrollbar.
use iced_core::{Background, Color};

/// The appearance of a scrollable.
#[derive(Debug, Clone, Copy)]
pub struct Scrollbar {
    pub background: Option<Background>,
    pub border_radius: f32,
    pub border_width: f32,
    pub border_color: Color,
    pub scroller: Scroller,
}

/// The appearance of the scroller of a scrollable.
#[derive(Debug, Clone, Copy)]
pub struct Scroller {
    pub color: Color,
    pub border_radius: f32,
    pub border_width: f32,
    pub border_color: Color,
}

/// A set of rules that dictate the style of a scrollable.
pub trait StyleSheet {
    type Style: Default + Copy;

    /// Produces the style of an active scrollbar.
    fn active(&self, style: Self::Style) -> Scrollbar;

    /// Produces the style of an hovered scrollbar.
    fn hovered(&self, style: Self::Style) -> Scrollbar;

    /// Produces the style of a scrollbar that is being dragged.
    fn dragging(&self, style: Self::Style) -> Scrollbar {
        self.hovered(style)
    }
}
