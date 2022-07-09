//! Show toggle controls using checkboxes.
use iced_core::{Background, Color};

/// The appearance of a checkbox.
#[derive(Debug, Clone, Copy)]
pub struct Appearance {
    pub background: Background,
    pub checkmark_color: Color,
    pub border_radius: f32,
    pub border_width: f32,
    pub border_color: Color,
    pub text_color: Option<Color>,
}

/// A set of rules that dictate the style of a checkbox.
pub trait StyleSheet {
    type Style: Default + Copy;

    fn active(&self, style: Self::Style, is_checked: bool) -> Appearance;

    fn hovered(&self, style: Self::Style, is_checked: bool) -> Appearance;
}
