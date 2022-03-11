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
pub trait StyleSheet<ColorPalette> {
    fn get_style(
        &self,
        color_palette: &ColorPalette,
        is_focused: bool,
        is_mouse_over: bool,
    ) -> Style {
        if is_focused {
            self.focused(color_palette)
        } else if is_mouse_over {
            self.hovered(color_palette)
        } else {
            self.active(color_palette)
        }
    }

    fn get_text_color(
        &self,
        color_palette: &ColorPalette,
        is_empty: bool,
    ) -> Color {
        if is_empty {
            self.placeholder_color(color_palette)
        } else {
            self.value_color(color_palette)
        }
    }

    /// Produces the style of an active text input.
    fn active(&self, color_palette: &ColorPalette) -> Style;

    /// Produces the style of a focused text input.
    fn focused(&self, color_palette: &ColorPalette) -> Style;

    fn placeholder_color(&self, color_palette: &ColorPalette) -> Color;

    fn value_color(&self, color_palette: &ColorPalette) -> Color;

    fn selection_color(&self, color_palette: &ColorPalette) -> Color;

    /// Produces the style of an hovered text input.
    fn hovered(&self, color_palette: &ColorPalette) -> Style {
        self.focused(color_palette)
    }
}

struct Default;

impl StyleSheet<IcedColorPalette> for Default {
    fn active(&self, color_palette: &ColorPalette) -> Style {
        Style {
            background: color_palette.surface.into(),
            border_radius: 5.0,
            border_width: 1.0,
            border_color: color_palette.accent,
        }
    }

    fn focused(&self, color_palette: &ColorPalette) -> Style {
        Style {
            border_color: Color::from_rgb(
                color_palette.accent.r - 0.2,
                color_palette.accent.g - 0.2,
                color_palette.accent.b - 0.2,
            ),
            ..self.active(color_palette)
        }
    }

    fn placeholder_color(&self, color_palette: &ColorPalette) -> Color {
        color_palette.accent
    }

    fn value_color(&self, color_palette: &ColorPalette) -> Color {
        color_palette.needs_better_naming
    }

    fn selection_color(&self, color_palette: &ColorPalette) -> Color {
        color_palette.text_highlight
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
