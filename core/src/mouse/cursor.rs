use crate::{Point, Rectangle, Vector};

/// The mouse cursor state.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum Cursor {
    /// The cursor has a defined position.
    Available(Point),

    /// The cursor is currently unavailable (i.e. out of bounds or busy).
    #[default]
    Unavailable,
}

impl Cursor {
    /// Returns the absolute position of the [`Cursor`], if available.
    pub fn position(self) -> Option<Point> {
        match self {
            Cursor::Available(position) => Some(position),
            Cursor::Unavailable => None,
        }
    }

    /// Returns the absolute position of the [`Cursor`], if available and inside
    /// the given bounds.
    ///
    /// If the [`Cursor`] is not over the provided bounds, this method will
    /// return `None`.
    pub fn position_over(self, bounds: Rectangle) -> Option<Point> {
        self.position().filter(|p| bounds.contains(*p))
    }

    /// Returns the relative position of the [`Cursor`] inside the given bounds,
    /// if available.
    ///
    /// If the [`Cursor`] is not over the provided bounds, this method will
    /// return `None`.
    pub fn position_in(self, bounds: Rectangle) -> Option<Point> {
        self.position_over(bounds)
            .map(|p| p - Vector::new(bounds.x, bounds.y))
    }

    /// Returns the relative position of the [`Cursor`] from the given origin,
    /// if available.
    pub fn position_from(self, origin: Point) -> Option<Point> {
        self.position().map(|p| p - Vector::new(origin.x, origin.y))
    }

    /// Returns true if the [`Cursor`] is over the given `bounds`.
    pub fn is_over(self, bounds: Rectangle) -> bool {
        self.position_over(bounds).is_some()
    }
}
