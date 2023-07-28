use crate::gradient::{self, Gradient};
use crate::Color;

/// The background of some element.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Background {
    /// A solid color.
    Color(Color),
    /// Linearly interpolate between several colors.
    Gradient(Gradient),
    // TODO: Add image variant
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
