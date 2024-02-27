//! Change the appearance of an application.
use crate::core::Color;
use crate::theme;

/// A set of rules that dictate the style of an application.
pub trait StyleSheet {
    /// The supported style of the [`StyleSheet`].
    type Style: Default;

    /// Returns the [`Appearance`] of the application for the provided [`Style`].
    ///
    /// [`Style`]: Self::Style
    fn appearance(&self, style: &Self::Style) -> Appearance;

    /// Returns the [`theme::Palette`] of the application, if any.
    ///
    /// This may be used by other parts of the `iced` runtime to
    /// try to match the style of your application.
    ///
    /// For instance, the Iced Axe uses this [`theme::Palette`] to
    /// automatically style itself using your application's colors.
    fn palette(&self) -> Option<theme::Palette> {
        None
    }
}

/// The appearance of an application.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Appearance {
    /// The background [`Color`] of the application.
    pub background_color: Color,

    /// The default text [`Color`] of the application.
    pub text_color: Color,
}
