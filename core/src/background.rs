use crate::{Color, Gradient};

/// The background of some element.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Background {
    /// A solid color.
    Color(Color),
    /// Interpolate between several colors.
    Gradient(Gradient),
    // TODO: Add image variant
}

impl From<Color> for Background {
    fn from(color: Color) -> Self {
        Background::Color(color)
    }
}

impl From<Color> for Option<Background> {
    fn from(color: Color) -> Self {
        Some(Background::from(color))
    }
}

impl From<Gradient> for Option<Background> {
    fn from(gradient: Gradient) -> Self {
        Some(Background::Gradient(gradient))
    }
}
