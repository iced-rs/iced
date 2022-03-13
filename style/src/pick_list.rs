use crate::{menu, IcedTheme};
use iced_core::{Background, Color};

/// The appearance of a pick list.
#[derive(Debug, Clone, Copy)]
pub struct Style {
    pub text_color: Color,
    pub placeholder_color: Color,
    pub background: Background,
    pub border_radius: f32,
    pub border_width: f32,
    pub border_color: Color,
    pub icon_size: f32,
}

/// A set of rules that dictate the style of a container.
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

    fn menu_style(&self, theme: &Theme) -> menu::Style;

    fn active(&self, theme: &Theme) -> Style;

    /// Produces the style of a container.
    fn hovered(&self, theme: &Theme) -> Style;
}

struct Default;

impl StyleSheet<IcedTheme> for Default {
    fn menu_style(&self, theme: &Theme) -> menu::Style {
        let style_sheet: Box<dyn menu::StyleSheet<Theme>> =
            Default::default();
        style_sheet.style(theme)
    }

    fn active(&self, theme: &Theme) -> Style {
        Style {
            text_color: theme.text,
            placeholder_color: theme.needs_better_naming,
            background: theme.surface.into(),
            border_radius: 0.0,
            border_width: 1.0,
            border_color: theme.accent,
            icon_size: 0.7,
        }
    }

    fn hovered(&self, theme: &Theme) -> Style {
        Style {
            border_color: theme.hover,
            ..self.active(theme)
        }
    }
}

impl<'a> std::default::Default for Box<dyn StyleSheet<IcedTheme> + 'a> {
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
