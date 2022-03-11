//! Provide progress feedback to your users.
use crate::Theme;
use iced_core::Background;

/// The appearance of a progress bar.
#[derive(Debug, Clone, Copy)]
pub struct Style {
    pub background: Background,
    pub bar: Background,
    pub border_radius: f32,
}

/// A set of rules that dictate the style of a progress bar.
pub trait StyleSheet<ColorPalette> {
    fn style(&self, color_palette: &ColorPalette) -> Style;
}

struct Default;

impl StyleSheet<IcedColorPalette> for Default {
    fn style(&self, color_palette: &ColorPalette) -> Style {
        Style {
            background: color_palette.accent.into(),
            bar: color_palette.active.into(),
            border_radius: 5.0,
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
