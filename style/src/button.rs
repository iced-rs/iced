//! Allow your users to perform actions by pressing a button.
use crate::{IcedColorPalette, Theme};
use iced_core::{Background, Color, Vector};
use std::fmt::Debug;

/// The appearance of a button.
#[derive(Debug, Clone, Copy)]
pub struct Style {
    pub shadow_offset: Vector,
    pub background: Option<Background>,
    pub border_radius: f32,
    pub border_width: f32,
    pub border_color: Color,
}

/// A set of rules that dictate the style of a button.
pub trait StyleSheet<ColorPalette> {
    fn get_style(
        &self,
        color_palette: &ColorPalette,
        is_disabled: bool,
        is_mouse_over: bool,
        is_pressed: bool,
    ) -> Style {
        if is_disabled {
            self.disabled(color_palette)
        } else if is_mouse_over {
            if is_pressed {
                self.pressed(color_palette)
            } else {
                self.hovered(color_palette)
            }
        } else {
            self.active(color_palette)
        }
    }

    fn active(&self, color_palette: &ColorPalette) -> Style;

    fn hovered(&self, color_palette: &ColorPalette) -> Style;

    fn pressed(&self, color_palette: &ColorPalette) -> Style;

    fn disabled(&self, color_palette: &ColorPalette) -> Style;
}

struct Default;

impl StyleSheet<IcedColorPalette> for Default {
    fn active(&self, color_palette: &IcedColorPalette) -> Style {
        Style {
            shadow_offset: Vector::new(0.0, 0.0),
            background: Some(color_palette.surface.into()),
            border_radius: 2.0,
            border_width: 1.0,
            border_color: color_palette.accent,
        }
    }

    fn hovered(&self, color_palette: &IcedColorPalette) -> Style {
        let active = self.active(color_palette);

        Style {
            shadow_offset: active.shadow_offset + Vector::new(0.0, 1.0),
            ..active
        }
    }

    fn pressed(&self, color_palette: &IcedColorPalette) -> Style {
        Style {
            shadow_offset: Vector::default(),
            ..self.active(color_palette)
        }
    }

    fn disabled(&self, color_palette: &IcedColorPalette) -> Style {
        let active = self.active(color_palette);

        Style {
            shadow_offset: Vector::default(),
            background: active.background.map(|background| match background {
                Background::Color(color) => Background::Color(Color {
                    a: color.a * 0.5,
                    ..color
                }),
            }),
            ..active
        }
    }
}

impl<'a> std::default::Default for Box<dyn StyleSheet<IcedColorPalette> + 'a> {
    fn default() -> Self {
        Box::new(Default)
    }
}

impl<'a, T> From<T> for Box<dyn StyleSheet<IcedColorPalette> + 'a>
where
    T: StyleSheet<IcedColorPalette> + 'a,
{
    fn from(style_sheet: T) -> Self {
        Box::new(style_sheet)
    }
}
