//! Change the appearance of a checkbox.
use iced_core::{Background, Border, Color};

/// The appearance of a checkbox.
#[derive(Debug, Clone, Copy)]
pub struct Appearance {
    /// The [`Background`] of the checkbox.
    pub background: Background,
    /// The icon [`Color`] of the checkbox.
    pub icon_color: Color,
    /// The [`Border`] of hte checkbox.
    pub border: Border,
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

    /// Produces the disabled [`Appearance`] of a checkbox.
    fn disabled(&self, style: &Self::Style, is_checked: bool) -> Appearance {
        let active = self.active(style, is_checked);

        Appearance {
            background: match active.background {
                Background::Color(color) => Background::Color(Color {
                    a: color.a * 0.5,
                    ..color
                }),
                Background::Gradient(gradient) => {
                    Background::Gradient(gradient.mul_alpha(0.5))
                }
            },
            ..active
        }
    }
}
