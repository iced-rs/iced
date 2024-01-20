//! Change the appearance of a container.
use crate::core::{Background, Border, Color, Pixels, Shadow};

/// The appearance of a container.
#[derive(Debug, Clone, Copy, Default)]
pub struct Appearance {
    /// The text [`Color`] of the container.
    pub text_color: Option<Color>,
    /// The [`Background`] of the container.
    pub background: Option<Background>,
    /// The [`Border`] of the container.
    pub border: Border,
    /// The [`Shadow`] of the container.
    pub shadow: Shadow,
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
            border: Border {
                color: color.into(),
                width: width.into().0,
                ..Border::default()
            },
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

/// A set of rules that dictate the [`Appearance`] of a container.
pub trait StyleSheet {
    /// The supported style of the [`StyleSheet`].
    type Style: Default;

    /// Produces the [`Appearance`] of a container.
    fn appearance(&self, style: &Self::Style) -> Appearance;
}
