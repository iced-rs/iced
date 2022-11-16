//! Change the appearance of a svg.

use iced_core::Color;

/// The appearance of a svg.
#[derive(Debug, Default, Clone, Copy)]
pub struct Appearance {
    /// Changes the fill color
    ///
    /// Useful for coloring a symbolic icon.
    pub fill: Option<Color>,
}

/// The stylesheet of a svg.
pub trait StyleSheet {
    /// The supported style of the [`StyleSheet`].
    type Style: Default + Copy;

    /// Produces the [`Appearance`] of the svg.
    fn appearance(&self, style: Self::Style) -> Appearance;
}
