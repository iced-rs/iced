//! Show toggle controls using checkboxes.
use iced_core::{Background, Color};

/// The appearance of a checkbox.
#[derive(Debug, Clone, Copy)]
pub struct Style {
    pub background: Background,
    pub checkmark_color: Color,
    pub border_radius: f32,
    pub border_width: f32,
    pub border_color: Color,
}

/// A set of rules that dictate the style of a checkbox.
pub trait StyleSheet {
    fn active(&self, is_checked: bool) -> Style;

    fn hovered(&self, is_checked: bool) -> Style;
}

struct Default;

impl StyleSheet for Default {
    fn active(&self, _is_checked: bool) -> Style {
        Style {
            background: Background::Color(Color::from_rgb(0.95, 0.95, 0.95)),
            checkmark_color: Color::from_rgb(0.3, 0.3, 0.3),
            border_radius: 5.0,
            border_width: 1.0,
            border_color: Color::from_rgb(0.6, 0.6, 0.6),
        }
    }

    fn hovered(&self, is_checked: bool) -> Style {
        Style {
            background: Background::Color(Color::from_rgb(0.90, 0.90, 0.90)),
            ..self.active(is_checked)
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
