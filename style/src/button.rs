//! Allow your users to perform actions by pressing a button.
use iced_core::{Background, Color, Vector};

/// The appearance of a button.
#[derive(Debug, Clone, Copy)]
pub struct Style {
    pub shadow_offset: Vector,
    pub background: Option<Background>,
    pub border_radius: f32,
    pub border_width: f32,
    pub border_color: Color,
    pub text_color: Color,
}

impl std::default::Default for Style {
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
    type Variant;

    fn active(&self, variant: Self::Variant) -> Style;

    fn hovered(&self, variant: Self::Variant) -> Style {
        let active = self.active(variant);

        Style {
            shadow_offset: active.shadow_offset + Vector::new(0.0, 1.0),
            ..active
        }
    }

    fn pressed(&self, variant: Self::Variant) -> Style {
        Style {
            shadow_offset: Vector::default(),
            ..self.active(variant)
        }
    }

    fn disabled(&self, variant: Self::Variant) -> Style {
        let active = self.active(variant);

        Style {
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
