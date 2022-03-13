use crate::{IcedTheme};
use iced_core::{Background, Color};

/// The appearance of a menu.
#[derive(Debug, Clone, Copy)]
pub struct Style {
    pub text_color: Color,
    pub background: Background,
    pub border_width: f32,
    pub border_color: Color,
    pub selected_text_color: Color,
    pub selected_background: Background,
}

/// A set of rules that dictate the style of a menu.
pub trait StyleSheet<Theme> {
    /// Produces the style of a menu.
    fn style(&self, theme: &Theme) -> Style;
}

struct Default;

impl StyleSheet<IcedTheme> for Default {
    fn style(&self, theme: &Theme) -> Style {
        Style {
            text_color: theme.text,
            background: theme.surface.into(),
            border_width: 1.0,
            border_color: theme.accent,
            selected_text_color: theme.text.inverse(),
            selected_background: theme.highlight.into(),
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
