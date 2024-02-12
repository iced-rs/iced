//! Change the appearance of a scrollable.
use crate::container;
use crate::core::{Background, Border, Color};

/// The appearance of a scrolable.
#[derive(Debug, Clone, Copy)]
pub struct Appearance {
    /// The [`container::Appearance`] of a scrollable.
    pub container: container::Appearance,
    /// The [`Scrollbar`] appearance.
    pub scrollbar: Scrollbar,
    /// The [`Background`] of the gap between a horizontal and vertical scrollbar.
    pub gap: Option<Background>,
}

/// The appearance of the scrollbar of a scrollable.
#[derive(Debug, Clone, Copy)]
pub struct Scrollbar {
    /// The [`Background`] of a scrollbar.
    pub background: Option<Background>,
    /// The [`Border`] of a scrollbar.
    pub border: Border,
    /// The appearance of the [`Scroller`] of a scrollbar.
    pub scroller: Scroller,
}

/// The appearance of the scroller of a scrollable.
#[derive(Debug, Clone, Copy)]
pub struct Scroller {
    /// The [`Color`] of the scroller.
    pub color: Color,
    /// The [`Border`] of the scroller.
    pub border: Border,
}

/// A set of rules that dictate the style of a scrollable.
pub trait StyleSheet {
    /// The supported style of the [`StyleSheet`].
    type Style: Default;

    /// Produces the [`Appearance`] of an active scrollable.
    fn active(&self, style: &Self::Style) -> Appearance;

    /// Produces the [`Appearance`] of a scrollable when it is being hovered.
    fn hovered(
        &self,
        style: &Self::Style,
        is_mouse_over_scrollbar: bool,
    ) -> Appearance;

    /// Produces the [`Appearance`] of a scrollable when it is being dragged.
    fn dragging(&self, style: &Self::Style) -> Appearance {
        self.hovered(style, true)
    }
}
