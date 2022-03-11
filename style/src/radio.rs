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
pub trait StyleSheet<ColorPalette> {
    fn get_style(
        &self,
        color_palette: &ColorPalette,
        is_mouse_over: bool,
    ) -> Style {
        if is_mouse_over {
            self.hovered(color_palette)
        } else {
            self.active(color_palette)
        }
    }

    fn active(&self, color_palette: &ColorPalette) -> Style;

    fn hovered(&self, color_palette: &ColorPalette) -> Style;
}

struct Default;

impl StyleSheet<IcedColorPalette> for Default {
    fn active(&self, color_palette: &ColorPalette) -> Style {
        Style {
            background: color_palette.surface.into(),
            dot_color: color_palette.needs_better_naming,
            border_width: 1.0,
            border_color: color_palette.accent,
            text_color: Some(color_palette.text),
        }
    }

    fn hovered(&self, color_palette: &ColorPalette) -> Style {
        Style {
            background: color_palette.hover.into(),
            ..self.active(color_palette)
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
