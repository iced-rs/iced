//! Change the appearance of a checkbox.
use iced_core::{Background, Color};

/// The appearance of a checkbox.
#[derive(Debug, Clone, Copy)]
pub struct Appearance {
    /// The [`Background`] of the checkbox.
    pub background: Background,
    /// The checkmark [`Color`] of the checkbox.
    pub checkmark_color: Color,
    /// The border radius of the checkbox.
    pub border_radius: f32,
    /// The border width of the checkbox.
    pub border_width: f32,
    /// The border [`Color`] of the checkbox.
    pub border_color: Color,
    /// The text [`Color`] of the checkbox.
    pub text_color: Option<Color>,
}

/// A set of rules that dictate the style of a checkbox.
pub trait StyleSheet {
    /// The supported style of the [`StyleSheet`].
    type Style: Default;

    /// Produces the active [`Appearance`] of a checkbox.
    fn active(&self, style: &Self::Style, is_checked: bool) -> Appearance;

    /// Produces the hovered [`Appearance`] of a checkbox.
    fn hovered(&self, style: &Self::Style, is_checked: bool) -> Appearance;
}
