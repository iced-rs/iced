use crate::core::Color;
use crate::geometry::Gradient;

/// The coloring style of some drawing.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Style {
    /// A solid [`Color`].
    Solid(Color),

    /// A [`Gradient`] color.
    Gradient(Gradient),
}

impl From<Color> for Style {
    fn from(color: Color) -> Self {
        Self::Solid(color)
    }
}

impl From<Gradient> for Style {
    fn from(gradient: Gradient) -> Self {
        Self::Gradient(gradient)
    }
}
