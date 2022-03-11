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
pub trait StyleSheet<ColorPalette> {
    fn get_style(
        &self,
        color_palette: &ColorPalette,
        is_mouse_over: bool,
        is_active: bool,
    ) -> Style {
        if is_mouse_over {
            self.hovered(color_palette, is_active)
        } else {
            self.active(color_palette, is_active)
        }
    }

    fn active(&self, color_palette: &ColorPalette, is_active: bool) -> Style;

    fn hovered(&self, color_palette: &ColorPalette, is_active: bool) -> Style;
}

struct Default;

impl StyleSheet<IcedColorPalette> for Default {
    fn active(&self, color_palette: &ColorPalette, is_active: bool) -> Style {
        Style {
            background: if is_active {
                color_palette.accent
            } else {
                color_palette.surface
            },
            background_border: None,
            foreground: color_palette.surface,
            foreground_border: None,
        }
    }

    fn hovered(&self, color_palette: &ColorPalette, is_active: bool) -> Style {
        Style {
            foreground: Color::from_rgb(
                color_palette.surface.r - 0.05,
                color_palette.surface.g - 0.05,
                color_palette.surface.b - 0.05,
            ),
            ..self.active(color_palette, is_active)
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
