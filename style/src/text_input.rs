//! Display fields that can be filled with text.
use crate::Theme;
use iced_core::{Background, Color};

/// The appearance of a text input.
#[derive(Debug, Clone, Copy)]
pub struct Style {
    pub background: Background,
    pub border_radius: f32,
    pub border_width: f32,
    pub border_color: Color,
}

/// A set of rules that dictate the style of a text input.
pub trait StyleSheet {
    type Theme;
    fn get_style(
        &self,
        theme: &Self::Theme,
        is_focused: bool,
        is_mouse_over: bool,
    ) -> Style {
        if is_focused {
            self.focused(theme)
        } else if is_mouse_over {
            self.hovered(theme)
        } else {
            self.active(theme)
        }
    }

    fn get_text_color(&self, theme: &Self::Theme, is_empty: bool) -> Color {
        if is_empty {
            self.placeholder_color(theme)
        } else {
            self.value_color(theme)
        }
    }

    /// Produces the style of an active text input.
    fn active(&self, theme: &Self::Theme) -> Style;

    /// Produces the style of a focused text input.
    fn focused(&self, theme: &Self::Theme) -> Style;

    fn placeholder_color(&self, theme: &Self::Theme) -> Color;

    fn value_color(&self, theme: &Self::Theme) -> Color;

    fn selection_color(&self, theme: &Self::Theme) -> Color;

    /// Produces the style of an hovered text input.
    fn hovered(&self, theme: &Self::Theme) -> Style {
        self.focused(theme)
    }
}

struct Default;

impl StyleSheet<IcedTheme> for Default {
    fn active(&self, theme: &Self::Theme) -> Style {
        Style {
            background: theme.surface.into(),
            border_radius: 5.0,
            border_width: 1.0,
            border_color: theme.accent,
        }
    }

    fn focused(&self, theme: &Self::Theme) -> Style {
        Style {
            border_color: Color::from_rgb(
                theme.accent.r - 0.2,
                theme.accent.g - 0.2,
                theme.accent.b - 0.2,
            ),
            ..self.active(theme)
        }
    }

    fn placeholder_color(&self, theme: &Self::Theme) -> Color {
        theme.accent
    }

    fn value_color(&self, theme: &Self::Theme) -> Color {
        theme.needs_better_naming
    }

    fn selection_color(&self, theme: &Self::Theme) -> Color {
        theme.text_highlight
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
