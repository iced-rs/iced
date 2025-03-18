//! Handle mouse events.
pub mod click;

mod button;
mod event;
mod interaction;

pub use button::Button;
pub use click::Click;
pub use event::{Event, ScrollDelta};
pub use interaction::Interaction;

use crate::{Point, Rectangle, Transformation, Vector};

/// The mouse state.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum Mouse {
    /// The mouse has a defined position.
    Available(Point),

    /// The mouse has a defined position, but it's levitating over a layer above.
    Levitating(Point),

    /// The mouse is currently unavailable (i.e. out of bounds or busy).
    #[default]
    Unavailable,
}

impl Mouse {
    /// Returns the absolute position of the [`Mouse`], if available.
    pub fn position(self) -> Option<Point> {
        match self {
            Mouse::Available(position) => Some(position),
            Mouse::Levitating(_) | Mouse::Unavailable => None,
        }
    }

    /// Returns the absolute position of the [`Mouse`], if available and inside
    /// the given bounds.
    ///
    /// If the [`Mouse`] is not over the provided bounds, this method will
    /// return `None`.
    pub fn position_over(self, bounds: Rectangle) -> Option<Point> {
        self.position().filter(|p| bounds.contains(*p))
    }

    /// Returns the relative position of the [`Mouse`] inside the given bounds,
    /// if available.
    ///
    /// If the [`Mouse`] is not over the provided bounds, this method will
    /// return `None`.
    pub fn position_in(self, bounds: Rectangle) -> Option<Point> {
        self.position_over(bounds)
            .map(|p| p - Vector::new(bounds.x, bounds.y))
    }

    /// Returns the relative position of the [`Mouse`] from the given origin,
    /// if available.
    pub fn position_from(self, origin: Point) -> Option<Point> {
        self.position().map(|p| p - Vector::new(origin.x, origin.y))
    }

    /// Returns true if the [`Mouse`] is over the given `bounds`.
    pub fn is_over(self, bounds: Rectangle) -> bool {
        self.position_over(bounds).is_some()
    }

    /// Returns true if the [`Mouse`] is levitating over a layer above.
    pub fn is_levitating(self) -> bool {
        matches!(self, Self::Levitating(_))
    }

    /// Makes the [`Mouse`] levitate over a layer above.
    pub fn levitate(self) -> Self {
        match self {
            Self::Available(position) => Self::Levitating(position),
            _ => self,
        }
    }

    /// Brings the [`Mouse`] back to the current layer.
    pub fn land(self) -> Self {
        match self {
            Mouse::Levitating(position) => Mouse::Available(position),
            _ => self,
        }
    }
}

impl std::ops::Mul<Transformation> for Mouse {
    type Output = Self;

    fn mul(self, transformation: Transformation) -> Self {
        match self {
            Self::Available(position) => {
                Self::Available(position * transformation)
            }
            Self::Levitating(position) => {
                Self::Levitating(position * transformation)
            }
            Self::Unavailable => Self::Unavailable,
        }
    }
}
