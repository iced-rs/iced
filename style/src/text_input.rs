//! Display fields that can be filled with text.
use iced_core::{Background, Color};

/// The appearance of a text input.
#[derive(Debug, Clone, Copy)]
pub struct Appearance {
    pub background: Background,
    pub border_radius: f32,
    pub border_width: f32,
    pub border_color: Color,
}

/// A set of rules that dictate the style of a text input.
pub trait StyleSheet {
    type Style: Default + Copy;

    /// Produces the style of an active text input.
    fn active(&self, style: Self::Style) -> Appearance;

    /// Produces the style of a focused text input.
    fn focused(&self, style: Self::Style) -> Appearance;

    fn placeholder_color(&self, style: Self::Style) -> Color;

    fn value_color(&self, style: Self::Style) -> Color;

    fn selection_color(&self, style: Self::Style) -> Color;

    /// Produces the style of an hovered text input.
    fn hovered(&self, style: Self::Style) -> Appearance {
        self.focused(style)
    }
}
