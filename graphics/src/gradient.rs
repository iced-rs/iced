//! For creating a Gradient.
mod linear;

pub use crate::gradient::linear::Linear;
use crate::widget::canvas::frame::Transform;
use crate::{Point, Color};

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

    /// Modifies the start & end stops of the gradient to have a proper transform value.
    pub(crate) fn transform(mut self, transform: &Transform) -> Self {
        match &mut self {
            Gradient::Linear(linear) => {
                linear.start = transform.apply_to(linear.start);
                linear.end = transform.apply_to(linear.end);
            }
        }
        self
    }
}
