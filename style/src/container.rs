//! Change the appearance of a container.
use crate::core::{Background, BorderRadius, Color, Pixels};

/// The appearance of a container.
#[derive(Debug, Clone, Copy)]
pub struct Appearance {
    /// The text [`Color`] of the container.
    pub text_color: Option<Color>,
    /// The [`Background`] of the container.
    pub background: Option<Background>,
    /// The border radius of the container.
    pub border_radius: BorderRadius,
    /// The border width of the container.
    pub border_width: f32,
    /// The border [`Color`] of the container.
    pub border_color: Color,
}

impl Appearance {
    /// Derives a new [`Appearance`] with a border of the given [`Color`] and
    /// `width`.
    pub fn with_border(
        self,
        color: impl Into<Color>,
        width: impl Into<Pixels>,
    ) -> Self {
        Self {
            border_color: color.into(),
            border_width: width.into().0,
            ..self
        }
    }

    /// Derives a new [`Appearance`] with the given [`Background`].
    pub fn with_background(self, background: impl Into<Background>) -> Self {
        Self {
            background: Some(background.into()),
            ..self
        }
    }
}

impl std::default::Default for Appearance {
    fn default() -> Self {
        Self {
            text_color: None,
            background: None,
            border_radius: 0.0.into(),
            border_width: 0.0,
            border_color: Color::TRANSPARENT,
        }
    }
}

/// A set of rules that dictate the [`Appearance`] of a container.
pub trait StyleSheet {
    /// The supported style of the [`StyleSheet`].
    type Style: Default;

    /// Produces the [`Appearance`] of a container.
    fn appearance(&self, style: &Self::Style) -> Appearance;
}
