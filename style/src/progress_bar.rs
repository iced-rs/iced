//! Provide progress feedback to your users.
use iced_core::{Background, Color};

/// The appearance of a progress bar.
#[derive(Debug, Clone)]
pub struct Style {
    pub background: Background,
    pub bar: Background,
    pub border_radius: f32,
}

/// A set of rules that dictate the style of a progress bar.
pub trait StyleSheet {
    fn style(&self) -> Style;
}

struct Default;

impl StyleSheet for Default {
    fn style(&self) -> Style {
        Style {
            background: Background::Color(Color::from_rgb(0.6, 0.6, 0.6)),
            bar: Background::Color(Color::from_rgb(0.3, 0.9, 0.3)),
            border_radius: 5.0,
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
    T: 'a + StyleSheet,
{
    fn from(style: T) -> Self {
        Box::new(style)
    }
}
