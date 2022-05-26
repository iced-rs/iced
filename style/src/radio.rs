//! Create choices using radio buttons.
use iced_core::{Background, Color};

/// The appearance of a radio button.
#[derive(Debug, Clone, Copy)]
pub struct Appearance {
    pub background: Background,
    pub dot_color: Color,
    pub border_width: f32,
    pub border_color: Color,
    pub text_color: Option<Color>,
}

/// A set of rules that dictate the style of a radio button.
pub trait StyleSheet {
    type Style: Default + Copy;

    fn active(&self, style: Self::Style) -> Appearance;

    fn hovered(&self, style: Self::Style) -> Appearance;
}
