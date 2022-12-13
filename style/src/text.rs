//! Change the appearance of text.
use iced_core::Color;

/// The style sheet of some text.
pub trait StyleSheet {
    /// The supported style of the [`StyleSheet`].
    type Style: Default + Copy;

    /// Produces the [`Appearance`] of some text.
    fn appearance(&self, style: Self::Style) -> Appearance;
}

/// The apperance of some text.
#[derive(Debug, Clone, Copy, Default)]
pub struct Appearance {
    /// The [`Color`] of the text.
    ///
    /// The default, `None`, means using the inherited color.
    pub color: Option<Color>,
}
