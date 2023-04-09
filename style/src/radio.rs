//! Create choices using radio buttons.
use iced_core::{Background, Color};

/// The appearance of a radio button.
#[derive(Debug, Clone, Copy)]
pub struct Style {
    pub background: Background,
    pub dot_color: Color,
    pub border_width: f32,
    pub border_color: Color,
}

/// A set of rules that dictate the style of a radio button.
pub trait StyleSheet {
    fn active(&self) -> Style;

    fn hovered(&self) -> Style;
}

struct Default;

impl StyleSheet for Default {
    fn active(&self) -> Style {
        Style {
            background: Background::Color(Color::from_rgb(0.95, 0.95, 0.95)),
            dot_color: Color::from_rgb(0.3, 0.3, 0.3),
            border_width: 1.0,
            border_color: Color::from_rgb(0.6, 0.6, 0.6),
        }
    }

    fn hovered(&self) -> Style {
        Style {
            background: Background::Color(Color::from_rgb(0.90, 0.90, 0.90)),
            ..self.active()
        }
    }
}

impl std::default::Default for Box<dyn StyleSheet> {
    fn default() -> Self {
        Box::new(Default)
    }
}

impl<T> From<T> for Box<dyn StyleSheet>
where
    T: 'static + StyleSheet,
{
    fn from(style: T) -> Self {
        Box::new(style)
    }
}
