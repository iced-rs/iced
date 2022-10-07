//! For creating a Gradient.
mod linear;

pub use crate::gradient::linear::{Linear, Position, Location};
use crate::widget::canvas::frame::Transform;
use crate::Color;
use crate::widget::canvas::{Fill, fill};

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
    pub fn linear(position: impl Into<Position>) -> linear::Builder {
        linear::Builder::new(position.into())
    }

    /// Modifies the start & end stops of the gradient to have a proper transform value.
    pub(crate) fn transform(mut self, transform: &Transform) -> Self {
        match &mut self {
            Gradient::Linear(linear) => {
                linear.start = transform.transform_point(linear.start);
                linear.end = transform.transform_point(linear.end);
            }
        }
        self
    }
}

impl<'a> Into<Fill<'a>> for &'a Gradient {
    fn into(self) -> Fill<'a> {
        Fill {
            style: fill::Style::Gradient(self),
            .. Default::default()
        }
    }
}
