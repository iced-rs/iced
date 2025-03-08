use crate::Color;
use crate::gradient::{self, Gradient};

/// The background of some element.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Background {
    /// A solid color.
    Color(Color),
    /// Linearly interpolate between several colors.
    Gradient(Gradient),
    // TODO: Add image variant
}

impl Background {
    /// Scales the alpha channel of the [`Background`] by the given
    /// factor.
    pub fn scale_alpha(self, factor: f32) -> Self {
        match self {
            Self::Color(color) => Self::Color(color.scale_alpha(factor)),
            Self::Gradient(gradient) => {
                Self::Gradient(gradient.scale_alpha(factor))
            }
        }
    }
}

impl From<Color> for Background {
    fn from(color: Color) -> Self {
        Background::Color(color)
    }
}

impl From<Gradient> for Background {
    fn from(gradient: Gradient) -> Self {
        Background::Gradient(gradient)
    }
}

impl From<gradient::Linear> for Background {
    fn from(gradient: gradient::Linear) -> Self {
        Background::Gradient(Gradient::Linear(gradient))
    }
}
