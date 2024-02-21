//! Change the appearance of a svg.

use iced_core::Color;

/// The appearance of an SVG.
#[derive(Debug, Default, Clone, Copy)]
pub struct Appearance {
    /// The [`Color`] filter of an SVG.
    ///
    /// Useful for coloring a symbolic icon.
    ///
    /// `None` keeps the original color.
    pub color: Option<Color>,
}

/// The stylesheet of a svg.
pub trait StyleSheet {
    /// The supported style of the [`StyleSheet`].
    type Style: Default;

    /// Produces the [`Appearance`] of the svg.
    fn appearance(&self, style: &Self::Style) -> Appearance;

    /// Produces the hovered [`Appearance`] of a svg content.
    fn hovered(&self, style: &Self::Style) -> Appearance {
        self.appearance(style)
    }
}
