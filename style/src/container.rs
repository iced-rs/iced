//! Decorate content and apply alignment.
use crate::{IcedColorPalette, Theme};
use iced_core::{Background, Color};
use std::fmt::Debug;

/// The appearance of a container.
#[derive(Debug, Clone, Copy)]
pub struct Style {
    pub text_color: Option<Color>,
    pub background: Option<Background>,
    pub border_radius: f32,
    pub border_width: f32,
    pub border_color: Color,
}

/// A set of rules that dictate the style of a container.
pub trait StyleSheet<ColorPalette> {
    /// Produces the style of a container.
    fn style(&self, color_palette: &ColorPalette) -> Style;
}

struct Default;

impl StyleSheet<IcedColorPalette> for Default {
    fn style(&self, color_palette: &ColorPalette) -> Style {
        Style {
            text_color: Some(color_palette.text),
            background: None,
            border_radius: 0.0,
            border_width: 0.0,
            border_color: Color::TRANSPARENT,
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
