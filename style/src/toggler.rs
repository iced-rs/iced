//! Show toggle controls using togglers.
use crate::Theme;
use iced_core::Color;

/// The appearance of a toggler.
#[derive(Debug)]
pub struct Style {
    pub background: Color,
    pub background_border: Option<Color>,
    pub foreground: Color,
    pub foreground_border: Option<Color>,
}

/// A set of rules that dictate the style of a toggler.
pub trait StyleSheet {
    fn get_style(
        &self,
        theme: &Theme,
        is_mouse_over: bool,
        is_active: bool,
    ) -> Style {
        if is_mouse_over {
            self.hovered(theme, is_active)
        } else {
            self.active(theme, is_active)
        }
    }

    fn active(&self, theme: &Theme, is_active: bool) -> Style;

    fn hovered(&self, theme: &Theme, is_active: bool) -> Style;
}

struct Default;

impl StyleSheet for Default {
    fn active(&self, theme: &Theme, is_active: bool) -> Style {
        Style {
            background: if is_active {
                theme.accent
            } else {
                theme.surface
            },
            background_border: None,
            foreground: theme.surface,
            foreground_border: None,
        }
    }

    fn hovered(&self, theme: &Theme, is_active: bool) -> Style {
        Style {
            foreground: Color::from_rgb(
                theme.surface.r - 0.05,
                theme.surface.g - 0.05,
                theme.surface.b - 0.05,
            ),
            ..self.active(theme, is_active)
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
