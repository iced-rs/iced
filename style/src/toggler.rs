//! Show toggle controls using togglers.
use iced_core::Color;

/// The appearance of a toggler.
#[derive(Debug)]
pub struct Appearance {
    pub background: Color,
    pub background_border: Option<Color>,
    pub foreground: Color,
    pub foreground_border: Option<Color>,
}

/// A set of rules that dictate the style of a toggler.
pub trait StyleSheet {
    type Style: Default + Copy;

    fn active(&self, style: Self::Style, is_active: bool) -> Appearance;

    fn hovered(&self, style: Self::Style, is_active: bool) -> Appearance;
}
