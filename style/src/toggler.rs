//! Change the appearance of a toggler.
use iced_core::Color;

/// The appearance of a toggler.
#[derive(Debug, Clone, Copy)]
pub struct Appearance {
    /// The background [`Color`] of the toggler.
    pub background: Color,
    /// The [`Color`] of the background border of the toggler.
    pub background_border: Option<Color>,
    /// The foreground [`Color`] of the toggler.
    pub foreground: Color,
    /// The [`Color`] of the foreground border of the toggler.
    pub foreground_border: Option<Color>,
}

/// A set of rules that dictate the style of a toggler.
pub trait StyleSheet {
    /// The supported style of the [`StyleSheet`].
    type Style: Default;

    /// Returns the active [`Appearance`] of the toggler for the provided [`Style`].
    ///
    /// [`Style`]: Self::Style
    fn active(&self, style: &Self::Style, is_active: bool) -> Appearance;

    /// Returns the hovered [`Appearance`] of the toggler for the provided [`Style`].
    ///
    /// [`Style`]: Self::Style
    fn hovered(&self, style: &Self::Style, is_active: bool) -> Appearance;
}
