//! Change the appearance of an application.
use iced_core::Color;

/// A set of rules that dictate the style of an application.
pub trait StyleSheet {
    /// The supported style of the [`StyleSheet`].
    type Style: Default;

    /// Returns the [`Appearance`] of the application for the provided [`Style`].
    ///
    /// [`Style`]: Self::Style
    fn appearance(&self, style: &Self::Style) -> Appearance;
}

/// The appearance of an application.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Appearance {
    /// The background [`Color`] of the application.
    pub background_color: Color,

    /// The default text [`Color`] of the application.
    pub text_color: Color,
}
