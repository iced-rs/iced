//! For creating a Gradient.
pub mod linear;

pub use linear::Linear;

use crate::{Color, Point, Size};

#[derive(Debug, Clone, PartialEq)]
/// A fill which transitions colors progressively along a direction, either linearly, radially (TBD),
/// or conically (TBD).
pub enum Gradient {
    /// A linear gradient interpolates colors along a direction from its `start` to its `end`
    /// point.
    Linear(Linear),
}

impl Gradient {
    /// Creates a new linear [`linear::Builder`].
    pub fn linear(position: impl Into<Position>) -> linear::Builder {
        linear::Builder::new(position.into())
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
/// A point along the gradient vector where the specified [`color`] is unmixed.
///
/// [`color`]: Self::color
pub struct ColorStop {
    /// Offset along the gradient vector.
    pub offset: f32,

    /// The color of the gradient at the specified [`offset`].
    ///
    /// [`offset`]: Self::offset
    pub color: Color,
}

#[derive(Debug)]
/// The position of the gradient within its bounds.
pub enum Position {
    /// The gradient will be positioned with respect to two points.
    Absolute {
        /// The starting point of the gradient.
        start: Point,
        /// The ending point of the gradient.
        end: Point,
    },
    /// The gradient will be positioned relative to the provided bounds.
    Relative {
        /// The top left position of the bounds.
        top_left: Point,
        /// The width & height of the bounds.
        size: Size,
        /// The start [Location] of the gradient.
        start: Location,
        /// The end [Location] of the gradient.
        end: Location,
    },
}

impl From<(Point, Point)> for Position {
    fn from((start, end): (Point, Point)) -> Self {
        Self::Absolute { start, end }
    }
}

#[derive(Debug)]
/// The location of a relatively-positioned gradient.
pub enum Location {
    /// Top left.
    TopLeft,
    /// Top.
    Top,
    /// Top right.
    TopRight,
    /// Right.
    Right,
    /// Bottom right.
    BottomRight,
    /// Bottom.
    Bottom,
    /// Bottom left.
    BottomLeft,
    /// Left.
    Left,
}

impl Location {
    fn to_absolute(&self, top_left: Point, size: Size) -> Point {
        match self {
            Location::TopLeft => top_left,
            Location::Top => {
                Point::new(top_left.x + size.width / 2.0, top_left.y)
            }
            Location::TopRight => {
                Point::new(top_left.x + size.width, top_left.y)
            }
            Location::Right => Point::new(
                top_left.x + size.width,
                top_left.y + size.height / 2.0,
            ),
            Location::BottomRight => {
                Point::new(top_left.x + size.width, top_left.y + size.height)
            }
            Location::Bottom => Point::new(
                top_left.x + size.width / 2.0,
                top_left.y + size.height,
            ),
            Location::BottomLeft => {
                Point::new(top_left.x, top_left.y + size.height)
            }
            Location::Left => {
                Point::new(top_left.x, top_left.y + size.height / 2.0)
            }
        }
    }
}
