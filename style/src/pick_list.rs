use crate::menu;
use iced_core::{Background, Color};

/// The appearance of a pick list.
#[derive(Debug, Clone)]
pub struct Style {
    pub text_color: Color,
    pub placeholder_color: Color,
    pub background: Background,
    pub border_radius: f32,
    pub border_width: f32,
    pub border_color: Color,
    pub icon_size: f32,
}

impl std::default::Default for Style {
    fn default() -> Self {
        Self {
            text_color: Color::BLACK,
            placeholder_color: [0.4, 0.4, 0.4].into(),
            background: Background::Color([0.87, 0.87, 0.87].into()),
            border_radius: 0.0,
            border_width: 1.0,
            border_color: [0.7, 0.7, 0.7].into(),
            icon_size: 0.7,
        }
    }
}

/// A set of rules that dictate the style of a container.
pub trait StyleSheet {
    fn menu(&self) -> menu::Style;

    fn active(&self) -> Style;

    /// Produces the style of a container.
    fn hovered(&self) -> Style;
}

struct Default;

impl StyleSheet for Default {
    fn menu(&self) -> menu::Style {
        menu::Style::default()
    }

    fn active(&self) -> Style {
        Style::default()
    }

    fn hovered(&self) -> Style {
        Style {
            border_color: Color::BLACK,
            ..self.active()
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
