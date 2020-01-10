//! Decorate content and apply alignment.
use iced_core::{Background, Color};

/// The appearance of a container.
#[derive(Debug, Clone, Copy)]
pub struct Style {
    pub text_color: Option<Color>,
    pub background: Option<Background>,
    pub border_radius: u16,
    pub border_width: u16,
    pub border_color: Color,
}

impl std::default::Default for Style {
    fn default() -> Self {
        Self {
            text_color: None,
            background: None,
            border_radius: 0,
            border_width: 0,
            border_color: Color::TRANSPARENT,
        }
    }
}

/// A set of rules that dictate the style of a container.
pub trait StyleSheet {
    /// Produces the style of a container.
    fn style(&self) -> Style;
}

struct Default;

impl StyleSheet for Default {
    fn style(&self) -> Style {
        Style {
            text_color: None,
            background: None,
            border_radius: 0,
            border_width: 0,
            border_color: Color::TRANSPARENT,
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
