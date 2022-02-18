//! Show toggle controls using checkboxes.
use iced_core::{Background, Color};

/// The appearance of a checkbox.
#[derive(Debug, Clone)]
pub struct Style {
    pub background: Background,
    pub checkmark_color: Color,
    pub border_radius: f32,
    pub border_width: f32,
    pub border_color: Color,
    pub text_color: Option<Color>,
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
            text_color: None,
        }
    }

    fn hovered(&self, is_checked: bool) -> Style {
        Style {
            background: Background::Color(Color::from_rgb(0.90, 0.90, 0.90)),
            ..self.active(is_checked)
        }
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
