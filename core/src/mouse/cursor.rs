use crate::{Point, Rectangle, Transformation, Vector};

/// The mouse cursor state.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum Cursor {
    /// The cursor has a defined position.
    Available(Point),

    /// The cursor has a defined position, but it's levitating over a layer above.
    ///
    /// You can [`land`](Self::land) and [`observe`](Self::observe) it safely,
    /// and make it [`Available`](Self::Available) again.
    Levitating(Point),

    /// The cursor has a defined position, but it's obstructed by a layer above.
    ///
    /// It cannot [`land`](Self::land), but you can [`observe`](Self::observe) it from below.
    Obstructed(Point),

    /// The cursor is currently unavailable (i.e. out of bounds or busy).
    #[default]
    Unavailable,
}

impl Cursor {
    /// Returns the absolute position of the [`Cursor`], if available.
    pub fn position(self) -> Option<Point> {
        match self {
            Cursor::Available(position) => Some(position),
            Cursor::Levitating(_) | Cursor::Obstructed(_) | Cursor::Unavailable => None,
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

    /// Makes the [`Cursor`] obstructed over a layer above.
    pub fn obstruct(self) -> Self {
        match self {
            Self::Available(position) => Self::Obstructed(position),
            _ => self,
        }
    }

    /// If the [`Cursor`] is levitating, it makes it available again.
    pub fn land(self) -> Self {
        match self {
            Cursor::Levitating(position) => Cursor::Available(position),
            _ => self,
        }
    }

    /// If the [`Cursor`] is either levitating or obstructed, it makes it available again.
    pub fn observe(self) -> Self {
        match self {
            Cursor::Obstructed(position) | Cursor::Levitating(position) => {
                Cursor::Available(position)
            }
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
            Cursor::Obstructed(point) => Cursor::Obstructed(point + translation),
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
            Cursor::Obstructed(point) => Cursor::Obstructed(point - translation),
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
            Self::Obstructed(position) => Self::Obstructed(position * transformation),
            Self::Unavailable => Self::Unavailable,
        }
    }
}
