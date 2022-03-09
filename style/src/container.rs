//! Decorate content and apply alignment.
use crate::Theme;
use iced_core::{Background, Color};
use std::fmt::Debug;

/// The appearance of a container.
#[derive(Debug, Clone, Copy)]
pub struct Style {
    pub text_color: Option<Color>,
    pub background: Option<Background>,
    pub border_radius: f32,
    pub border_width: f32,
    pub border_color: Color,
}

/// A set of rules that dictate the style of a container.
pub trait StyleSheet {
    /// Produces the style of a container.
    fn style(&self, theme: &Theme) -> Style;
}

struct Default;

impl StyleSheet for Default {
    fn style(&self, theme: &Theme) -> Style {
        Style {
            text_color: Some(theme.text),
            background: None,
            border_radius: 0.0,
            border_width: 0.0,
            border_color: Color::TRANSPARENT,
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
