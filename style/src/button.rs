//! Change the apperance of a button.
use std::time::Instant;

use iced_core::{Background, Color, Vector};

/// The appearance of a button.
#[derive(Debug, Clone, Copy)]
pub struct Appearance {
    /// The amount of offset to apply to the shadow of the button.
    pub shadow_offset: Vector,
    /// The [`Background`] of the button.
    pub background: Option<Background>,
    /// The border radius of the button.
    pub border_radius: f32,
    /// The border width of the button.
    pub border_width: f32,
    /// The border [`Color`] of the button.
    pub border_color: Color,
    /// The text [`Color`] of the button.
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

#[derive(Debug, Clone, Copy, PartialEq, Default)]
/// Direction of the animation
pub enum AnimationDirection {
    #[default]
    /// The animation goes forward
    Forward,
    /// The animation goes backward
    Backward,
}

#[derive(Debug, Clone, Copy, PartialEq)]
/// Hover animation
pub struct Hover {
    /// Animation direction: forward means it goes from non-hovered to hovered state
    pub direction: AnimationDirection,
    /// The instant the animation was started at
    pub started_at: Instant,
    /// The progress of the animationn, between 0.0 and 1.0
    pub animation_progress: f32,
    /// The progress the animation has been started at
    pub initial_progress: f32,
}

/// A set of rules that dictate the style of a button.
pub trait StyleSheet {
    /// The supported style of the [`StyleSheet`].
    type Style: Default;

    /// Produces the active [`Appearance`] of a button.
    fn active(&self, style: &Self::Style) -> Appearance;

    /// Produces the hovered [`Appearance`] of a button.
    fn hovered(
        &self,
        style: &Self::Style,
        _hover: Option<Hover>,
    ) -> Appearance {
        let active = self.active(style);

        Appearance {
            shadow_offset: active.shadow_offset + Vector::new(0.0, 1.0),
            ..active
        }
    }

    /// Produces the pressed [`Appearance`] of a button.
    fn pressed(&self, style: &Self::Style) -> Appearance {
        Appearance {
            shadow_offset: Vector::default(),
            ..self.active(style)
        }
    }

    /// Produces the disabled [`Appearance`] of a button.
    fn disabled(&self, style: &Self::Style) -> Appearance {
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
