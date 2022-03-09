//! Allow your users to perform actions by pressing a button.
use crate::Theme;
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
pub trait StyleSheet {
    fn get_style(
        &self,
        theme: &Theme,
        is_disabled: bool,
        is_mouse_over: bool,
        is_pressed: bool,
    ) -> Style {
        if is_disabled {
            self.disabled(theme)
        } else if is_mouse_over {
            if is_pressed {
                self.pressed(theme)
            } else {
                self.hovered(theme)
            }
        } else {
            self.active(theme)
        }
    }

    fn active(&self, theme: &Theme) -> Style;

    fn hovered(&self, theme: &Theme) -> Style {
        let active = self.active(theme);

        Style {
            shadow_offset: active.shadow_offset + Vector::new(0.0, 1.0),
            ..active
        }
    }

    fn pressed(&self, theme: &Theme) -> Style {
        Style {
            shadow_offset: Vector::default(),
            ..self.active(theme)
        }
    }

    fn disabled(&self, theme: &Theme) -> Style {
        let active = self.active(theme);

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

struct Default;

impl StyleSheet for Default {
    fn active(&self, theme: &Theme) -> Style {
        Style {
            shadow_offset: Vector::new(0.0, 0.0),
            background: Some(theme.surface.into()),
            border_radius: 2.0,
            border_width: 1.0,
            border_color: theme.accent,
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
