use crate::Vector;

/// A 2D point.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point {
    /// The X coordinate.
    pub x: f32,

    /// The Y coordinate.
    pub y: f32,
}

impl Point {
    /// The origin (i.e. a [`Point`] with both X=0 and Y=0).
    ///
    /// [`Point`]: struct.Point.html
    pub const ORIGIN: Point = Point::new(0.0, 0.0);

    /// Creates a new [`Point`] with the given coordinates.
    ///
    /// [`Point`]: struct.Point.html
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

impl From<[f32; 2]> for Point {
    fn from([x, y]: [f32; 2]) -> Self {
        Point { x, y }
    }
}

impl From<[u16; 2]> for Point {
    fn from([x, y]: [u16; 2]) -> Self {
        Point::new(x.into(), y.into())
    }
}

impl std::ops::Add<Vector> for Point {
    type Output = Self;

    fn add(self, vector: Vector) -> Self {
        Self {
            x: self.x + vector.x,
            y: self.y + vector.y,
        }
    }
}

impl std::ops::Sub<Vector> for Point {
    type Output = Self;

    fn sub(self, vector: Vector) -> Self {
        Self {
            x: self.x - vector.x,
            y: self.y - vector.y,
        }
    }
}
