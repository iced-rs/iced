use crate::{IcedColorPalette, Theme};
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
pub trait StyleSheet<ColorPalette> {
    /// Produces the style of a menu.
    fn style(&self, color_palette: &ColorPalette) -> Style;
}

struct Default;

impl StyleSheet<IcedColorPalette> for Default {
    fn style(&self, color_palette: &ColorPalette) -> Style {
        Style {
            text_color: color_palette.text,
            background: color_palette.surface.into(),
            border_width: 1.0,
            border_color: color_palette.accent,
            selected_text_color: color_palette.text.inverse(),
            selected_background: color_palette.highlight.into(),
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
