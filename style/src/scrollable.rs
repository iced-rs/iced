//! Change the appearance of a scrollable.
use iced_core::{Background, Color};

/// The appearance of a scrollable.
#[derive(Debug, Clone, Copy)]
pub struct Scrollbar {
    /// The [`Background`] of a scrollable.
    pub background: Option<Background>,
    /// The border radius of a scrollable.
    pub border_radius: f32,
    /// The border width of a scrollable.
    pub border_width: f32,
    /// The border [`Color`] of a scrollable.
    pub border_color: Color,
    /// The appearance of the [`Scroller`] of a scrollable.
    pub scroller: Scroller,
}

/// The appearance of the scroller of a scrollable.
#[derive(Debug, Clone, Copy)]
pub struct Scroller {
    /// The [`Color`] of the scroller.
    pub color: Color,
    /// The border radius of the scroller.
    pub border_radius: f32,
    /// The border width of the scroller.
    pub border_width: f32,
    /// The border [`Color`] of the scroller.
    pub border_color: Color,
}

/// A set of rules that dictate the style of a scrollable.
pub trait StyleSheet {
    /// The supported style of the [`StyleSheet`].
    type Style: Default;

    /// Produces the style of an active scrollbar.
    fn active(&self, style: &Self::Style) -> Scrollbar;

    /// Produces the style of a hovered scrollbar.
    fn hovered(&self, style: &Self::Style) -> Scrollbar;

    /// Produces the style of a scrollbar that is being dragged.
    fn dragging(&self, style: &Self::Style) -> Scrollbar {
        self.hovered(style)
    }

    /// Produces the style of an active horizontal scrollbar.
    fn active_horizontal(&self, style: &Self::Style) -> Scrollbar {
        self.active(style)
    }

    /// Produces the style of a hovered horizontal scrollbar.
    fn hovered_horizontal(&self, style: &Self::Style) -> Scrollbar {
        self.hovered(style)
    }

    /// Produces the style of a horizontal scrollbar that is being dragged.
    fn dragging_horizontal(&self, style: &Self::Style) -> Scrollbar {
        self.hovered_horizontal(style)
    }
}
