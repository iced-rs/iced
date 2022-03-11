//! Show toggle controls using checkboxes.
use crate::{IcedColorPalette, Theme};
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
pub trait StyleSheet<ColorPalette> {
    fn get_style(
        &self,
        color_palette: &ColorPalette,
        is_mouse_over: bool,
        is_checked: bool,
    ) -> Style {
        if is_mouse_over {
            self.hovered(color_palette, is_checked)
        } else {
            self.active(color_palette, is_checked)
        }
    }

    fn active(&self, color_palette: &ColorPalette, is_checked: bool) -> Style;

    fn hovered(&self, color_palette: &ColorPalette, is_checked: bool) -> Style;
}

struct Default;

impl StyleSheet<IcedColorPalette> for Default {
    fn active(&self, color_palette: &ColorPalette, _is_checked: bool) -> Style {
        Style {
            background: color_palette.surface.into(),
            checkmark_color: color_palette.needs_better_naming,
            border_radius: 5.0,
            border_width: 1.0,
            border_color: color_palette.accent,
            text_color: None,
        }
    }

    fn hovered(&self, color_palette: &ColorPalette, is_checked: bool) -> Style {
        Style {
            background: color_palette.hover.into(),
            ..self.active(color_palette, is_checked)
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
