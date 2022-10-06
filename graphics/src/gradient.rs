//! For creating a Gradient.
mod linear;

use iced_native::Color;
pub use crate::gradient::linear::Linear;
use crate::Point;

#[derive(Debug, Clone, PartialEq)]
/// A fill which transitions colors progressively along a direction, either linearly, radially (TBD),
/// or conically (TBD).
pub enum Gradient {
    /// A linear gradient interpolates colors along a direction from its [`start`] to its [`end`]
    /// point.
    Linear(Linear),
}

#[derive(Debug, Clone, Copy, PartialEq)]
/// A point along the gradient vector where the specified [`color`] is unmixed.
pub struct ColorStop {
    /// Offset along the gradient vector.
    pub offset: f32,
    /// The color of the gradient at the specified [`offset`].
    pub color: Color,
}

impl Gradient {
    /// Creates a new linear [`linear::Builder`].
    pub fn linear(start: Point, end: Point) -> linear::Builder {
        linear::Builder::new(start, end)
    }
}
