//! Let your users split regions of your application and organize layout
//! dynamically.
use iced_core::Color;

/// A set of rules that dictate the style of a container.
pub trait StyleSheet {
    type Style: Default + Copy;

    /// The [`Line`] to draw when a split is picked.
    fn picked_split(&self, style: Self::Style) -> Option<Line>;

    /// The [`Line`] to draw when a split is hovered.
    fn hovered_split(&self, style: Self::Style) -> Option<Line>;
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
