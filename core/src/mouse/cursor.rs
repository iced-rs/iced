use crate::{Point, Rectangle, Transformation, Vector};

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

impl Cursor {
    /// Returns the absolute position of the [`Cursor`], if available.
    pub fn position(self) -> Option<Point> {
        match self {
            Cursor::Available(position) => Some(position),
            Cursor::Levitating(_) | Cursor::Unavailable => None,
        }
    }

    /// Returns the absolute position of the [`Cursor`] while preserving levitating pointers.
    /// 返回 [`Cursor`] 的绝对位置，并在指针处于悬浮层状态时继续保留其坐标。
    pub fn position_including_levitation(self) -> Option<Point> {
        match self {
            Cursor::Available(position) | Cursor::Levitating(position) => Some(position),
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

    /// Returns the relative position of the [`Cursor`] from the given origin while
    /// preserving levitating pointers.
    /// 返回 [`Cursor`] 相对给定原点的位置，并在指针处于悬浮层状态时继续保留其坐标。
    pub fn position_from_including_levitation(self, origin: Point) -> Option<Point> {
        self.position_including_levitation()
            .map(|p| p - Vector::new(origin.x, origin.y))
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

#[cfg(test)]
mod tests {
    use super::Cursor;
    use crate::Point;

    /// Preserves levitating cursor coordinates through the dedicated drag helper.
    /// 通过专用拖拽辅助接口保留悬浮层光标的坐标。
    #[test]
    fn position_including_levitation_keeps_levitating_coordinates() {
        let cursor = Cursor::Levitating(Point::new(32.0, 48.0));

        assert_eq!(
            cursor.position_including_levitation(),
            Some(Point::new(32.0, 48.0))
        );
        assert_eq!(
            cursor.position_from_including_levitation(Point::new(2.0, 8.0)),
            Some(Point::new(30.0, 40.0))
        );
    }

    /// Keeps unavailable cursors unresolved even when levitating coordinates are requested.
    /// 即使请求悬浮层坐标，不可用光标仍然保持无法解析。
    #[test]
    fn position_including_levitation_keeps_unavailable_cursor_empty() {
        let cursor = Cursor::Unavailable;

        assert_eq!(cursor.position_including_levitation(), None);
        assert_eq!(
            cursor.position_from_including_levitation(Point::new(1.0, 1.0)),
            None
        );
    }
}
