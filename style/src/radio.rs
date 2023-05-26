//! Change the appearance of radio buttons.
use iced_core::{Background, Color};

use crate::animation;

/// The appearance of a radio button.
#[derive(Debug, Clone, Copy)]
pub struct Appearance {
    /// The [`Background`] of the radio button.
    pub background: Background,
    /// The [`Color`] of the dot of the radio button.
    pub dot_color: Color,
    /// The border width of the radio button.
    pub border_width: f32,
    /// The border [`Color`] of the radio button.
    pub border_color: Color,
    /// The text [`Color`] of the radio button.
    pub text_color: Option<Color>,
}

/// A set of rules that dictate the style of a radio button.
pub trait StyleSheet {
    /// The supported style of the [`StyleSheet`].
    type Style: Default;

    /// Produces the active [`Appearance`] of a radio button.
    fn active(
        &self,
        style: &Self::Style,
        is_selected: bool,
        pressed_animation: &animation::HoverPressedAnimation,
    ) -> Appearance;

    /// Produces the hovered [`Appearance`] of a radio button.
    fn hovered(
        &self,
        style: &Self::Style,
        is_selected: bool,
        hover_animation: &animation::HoverPressedAnimation,
    ) -> Appearance;
}
