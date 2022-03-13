//! Create choices using radio buttons.
use crate::Theme;
use iced_core::{Background, Color};

/// The appearance of a radio button.
#[derive(Debug, Clone, Copy)]
pub struct Style {
    pub background: Background,
    pub dot_color: Color,
    pub border_width: f32,
    pub border_color: Color,
    pub text_color: Option<Color>,
}

/// A set of rules that dictate the style of a radio button.
pub trait StyleSheet<Theme> {
    fn get_style(
        &self,
        theme: &Theme,
        is_mouse_over: bool,
    ) -> Style {
        if is_mouse_over {
            self.hovered(theme)
        } else {
            self.active(theme)
        }
    }

    fn active(&self, theme: &Theme) -> Style;

    fn hovered(&self, theme: &Theme) -> Style;
}

struct Default;

impl StyleSheet<IcedTheme> for Default {
    fn active(&self, theme: &Theme) -> Style {
        Style {
            background: theme.surface.into(),
            dot_color: theme.needs_better_naming,
            border_width: 1.0,
            border_color: theme.accent,
            text_color: Some(theme.text),
        }
    }

    fn hovered(&self, theme: &Theme) -> Style {
        Style {
            background: theme.hover.into(),
            ..self.active(theme)
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
