//! Change the appearance of a pane grid.
use iced_core::{Background, BorderRadius, Color};

/// The appearance of the hovered region of a pane grid.
#[derive(Debug, Clone, Copy)]
pub struct Appearance {
    /// The [`Background`] of the hovered pane region.
    pub background: Background,
    /// The border width of the hovered pane region.
    pub border_width: f32,
    /// The border [`Color`] of the hovered pane region.
    pub border_color: Color,
    /// The border radius of the hovered pane region.
    pub border_radius: BorderRadius,
}

/// A line.
///
/// It is normally used to define the highlight of something, like a split.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Line {
    /// The [`Color`] of the [`Line`].
    pub color: Color,

    /// The width of the [`Line`].
    pub width: f32,
}

/// A set of rules that dictate the style of a container.
pub trait StyleSheet {
    /// The supported style of the [`StyleSheet`].
    type Style: Default;

    /// The [`Appearance`] to draw when a pane is hovered.
    fn hovered_region(&self, style: &Self::Style) -> Appearance;

    /// The [`Line`] to draw when a split is picked.
    fn picked_split(&self, style: &Self::Style) -> Option<Line>;

    /// The [`Line`] to draw when a split is hovered.
    fn hovered_split(&self, style: &Self::Style) -> Option<Line>;
}
