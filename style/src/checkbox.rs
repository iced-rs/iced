//! Show toggle controls using checkboxes.
use crate::Theme;
use iced_core::{Background, Color};

/// The appearance of a checkbox.
#[derive(Debug, Clone, Copy)]
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
    fn get_style(
        &self,
        theme: &Theme,
        is_mouse_over: bool,
        is_checked: bool,
    ) -> Style {
        if is_mouse_over {
            self.hovered(theme, is_checked)
        } else {
            self.active(theme, is_checked)
        }
    }

    fn active(&self, theme: &Theme, is_checked: bool) -> Style;

    fn hovered(&self, theme: &Theme, is_checked: bool) -> Style;
}

struct Default;

impl StyleSheet for Default {
    fn active(&self, theme: &Theme, _is_checked: bool) -> Style {
        Style {
            background: theme.surface.into(),
            checkmark_color: theme.needs_better_naming,
            border_radius: 5.0,
            border_width: 1.0,
            border_color: theme.accent,
            text_color: None,
        }
    }

    fn hovered(&self, theme: &Theme, is_checked: bool) -> Style {
        Style {
            background: theme.hover.into(),
            ..self.active(theme, is_checked)
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
