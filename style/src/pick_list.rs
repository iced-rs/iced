use crate::{menu, IcedColorPalette, Theme};
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

    fn menu_style(&self, color_palette: &ColorPalette) -> menu::Style;

    fn active(&self, color_palette: &ColorPalette) -> Style;

    /// Produces the style of a container.
    fn hovered(&self, color_palette: &ColorPalette) -> Style;
}

struct Default;

impl StyleSheet<IcedColorPalette> for Default {
    fn menu_style(&self, color_palette: &ColorPalette) -> menu::Style {
        let style_sheet: Box<dyn menu::StyleSheet<ColorPalette>> =
            Default::default();
        style_sheet.style(color_palette)
    }

    fn active(&self, color_palette: &ColorPalette) -> Style {
        Style {
            text_color: color_palette.text,
            placeholder_color: color_palette.needs_better_naming,
            background: color_palette.surface.into(),
            border_radius: 0.0,
            border_width: 1.0,
            border_color: color_palette.accent,
            icon_size: 0.7,
        }
    }

    fn hovered(&self, color_palette: &ColorPalette) -> Style {
        Style {
            border_color: color_palette.hover,
            ..self.active(color_palette)
        }
    }
}

impl<'a> std::default::Default for Box<dyn StyleSheet + 'a> {
    fn default() -> Self {
        Box::new(DefaultStyle)
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
