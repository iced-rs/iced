use crate::{Point, PointerInput, Transformation, Vector};

/// The mouse cursor state.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum Cursor {
    /// The cursor has a defined position.
    Available(Point),

    /// The cursor has a defined position, but it's levitating over a layer above.
    Levitating(Point),

    /// The cursor is currently unavailable (i.e. out of bounds or busy).
    #[default]
    Unavailable,
}

impl PointerInput for Cursor {
    /// Returns the absolute position of the [`Cursor`], if available.
    fn position(self) -> Option<Point> {
        match self {
            Cursor::Available(position) => Some(position),
            Cursor::Levitating(_) | Cursor::Unavailable => None,
        }
    }
}

impl Cursor {
    /// Returns true if the [`Cursor`] is levitating over a layer above.
    pub fn is_levitating(self) -> bool {
        matches!(self, Self::Levitating(_))
    }

    /// Makes the [`Cursor`] levitate over a layer above.
    pub fn levitate(self) -> Self {
        match self {
            Self::Available(position) => Self::Levitating(position),
            _ => self,
        }
    }

    /// Brings the [`Cursor`] back to the current layer.
    pub fn land(self) -> Self {
        match self {
            Cursor::Levitating(position) => Cursor::Available(position),
            _ => self,
        }
    }
}

impl std::ops::Add<Vector> for Cursor {
    type Output = Self;

    fn add(self, translation: Vector) -> Self::Output {
        match self {
            Cursor::Available(point) => Cursor::Available(point + translation),
            Cursor::Levitating(point) => Cursor::Levitating(point + translation),
            Cursor::Unavailable => Cursor::Unavailable,
        }
    }
}

impl std::ops::Sub<Vector> for Cursor {
    type Output = Self;

    fn sub(self, translation: Vector) -> Self::Output {
        match self {
            Cursor::Available(point) => Cursor::Available(point - translation),
            Cursor::Levitating(point) => Cursor::Levitating(point - translation),
            Cursor::Unavailable => Cursor::Unavailable,
        }
    }
}

impl std::ops::Mul<Transformation> for Cursor {
    type Output = Self;

    fn mul(self, transformation: Transformation) -> Self {
        match self {
            Self::Available(position) => Self::Available(position * transformation),
            Self::Levitating(position) => Self::Levitating(position * transformation),
            Self::Unavailable => Self::Unavailable,
        }
    }
}
