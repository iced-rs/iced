//! Allow your users to perform actions by pressing a button.
use iced_core::{Background, Color, Vector};

/// The appearance of a button.
#[derive(Debug, Clone, Copy)]
pub struct Appearance {
    pub shadow_offset: Vector,
    pub background: Option<Background>,
    pub border_radius: f32,
    pub border_width: f32,
    pub border_color: Color,
    pub text_color: Color,
}

impl std::default::Default for Appearance {
    fn default() -> Self {
        Self {
            shadow_offset: Vector::default(),
            background: None,
            border_radius: 0.0,
            border_width: 0.0,
            border_color: Color::TRANSPARENT,
            text_color: Color::BLACK,
        }
    }
}

/// A set of rules that dictate the style of a button.
pub trait StyleSheet {
    type Style: Default + Copy;

    fn active(&self, style: Self::Style) -> Appearance;

    fn hovered(&self, style: Self::Style) -> Appearance {
        let active = self.active(style);

        Appearance {
            shadow_offset: active.shadow_offset + Vector::new(0.0, 1.0),
            ..active
        }
    }

    fn pressed(&self, style: Self::Style) -> Appearance {
        Appearance {
            shadow_offset: Vector::default(),
            ..self.active(style)
        }
    }

    fn disabled(&self, style: Self::Style) -> Appearance {
        let active = self.active(style);

        Appearance {
            shadow_offset: Vector::default(),
            background: active.background.map(|background| match background {
                Background::Color(color) => Background::Color(Color {
                    a: color.a * 0.5,
                    ..color
                }),
            }),
            text_color: Color {
                a: active.text_color.a * 0.5,
                ..active.text_color
            },
            ..active
        }
    }
}
