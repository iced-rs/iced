use iced_native::{Point, Rectangle};

/// The mouse cursor state.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Cursor {
    /// The cursor has a defined position.
    Available(Point),

    /// The cursor is currently unavailable (i.e. out of bounds or busy).
    Unavailable,
}

impl Cursor {
    // TODO: Remove this once this type is used in `iced_native` to encode
    // proper cursor availability
    pub(crate) fn from_window_position(position: Point) -> Self {
        if position.x < 0.0 || position.y < 0.0 {
            Cursor::Unavailable
        } else {
            Cursor::Available(position)
        }
    }

    /// Returns the absolute position of the [`Cursor`], if available.
    pub fn position(&self) -> Option<Point> {
        match self {
            Cursor::Available(position) => Some(*position),
            Cursor::Unavailable => None,
        }
    }

    /// Returns the relative position of the [`Cursor`] inside the given bounds,
    /// if available.
    ///
    /// If the [`Cursor`] is not over the provided bounds, this method will
    /// return `None`.
    pub fn position_in(&self, bounds: &Rectangle) -> Option<Point> {
        if self.is_over(bounds) {
            self.position_from(bounds.position())
        } else {
            None
        }
    }

    /// Returns the relative position of the [`Cursor`] from the given origin,
    /// if available.
    pub fn position_from(&self, origin: Point) -> Option<Point> {
        match self {
            Cursor::Available(position) => {
                Some(Point::new(position.x - origin.x, position.y - origin.y))
            }
            Cursor::Unavailable => None,
        }
    }

    /// Returns whether the [`Cursor`] is currently over the provided bounds
    /// or not.
    pub fn is_over(&self, bounds: &Rectangle) -> bool {
        match self {
            Cursor::Available(position) => bounds.contains(*position),
            Cursor::Unavailable => false,
        }
    }
}
