//! Display fields that can be filled with text.
use iced_core::{Background, Color};

/// The appearance of a text input.
#[derive(Debug, Clone)]
pub struct Style {
    pub background: Background,
    pub border_radius: f32,
    pub border_width: f32,
    pub border_color: Color,
}

impl std::default::Default for Style {
    fn default() -> Self {
        Self {
            background: Background::Color(Color::WHITE),
            border_radius: 0.0,
            border_width: 0.0,
            border_color: Color::TRANSPARENT,
        }
    }
}

/// A set of rules that dictate the style of a text input.
pub trait StyleSheet {
    /// Produces the style of an active text input.
    fn active(&self) -> Style;

    /// Produces the style of a focused text input.
    fn focused(&self) -> Style;

    fn placeholder_color(&self) -> Color;

    fn value_color(&self) -> Color;

    fn selection_color(&self) -> Color;

    /// Produces the style of an hovered text input.
    fn hovered(&self) -> Style {
        self.focused()
    }
}

struct Default;

impl StyleSheet for Default {
    fn active(&self) -> Style {
        Style {
            background: Background::Color(Color::WHITE),
            border_radius: 5.0,
            border_width: 1.0,
            border_color: Color::from_rgb(0.7, 0.7, 0.7),
        }
    }

    fn focused(&self) -> Style {
        Style {
            border_color: Color::from_rgb(0.5, 0.5, 0.5),
            ..self.active()
        }
    }

    fn placeholder_color(&self) -> Color {
        Color::from_rgb(0.7, 0.7, 0.7)
    }

    fn value_color(&self) -> Color {
        Color::from_rgb(0.3, 0.3, 0.3)
    }

    fn selection_color(&self) -> Color {
        Color::from_rgb(0.8, 0.8, 1.0)
    }
}

impl<'a> std::default::Default for Box<dyn StyleSheet + 'a> {
    fn default() -> Self {
        Box::new(Default)
    }
}

impl<'a, T> From<T> for Box<dyn StyleSheet + 'a>
where
    T: StyleSheet + 'a,
{
    fn from(style_sheet: T) -> Self {
        Box::new(style_sheet)
    }
}
