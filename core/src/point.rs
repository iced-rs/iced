use crate::Vector;

/// A 2D point.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Point {
    /// The X coordinate.
    pub x: f32,

    /// The Y coordinate.
    pub y: f32,
}

impl Point {
    /// The origin (i.e. a [`Point`] at (0, 0)).
    pub const ORIGIN: Point = Point::new(0.0, 0.0);

    /// Creates a new [`Point`] with the given coordinates.
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    /// Computes the distance to another [`Point`].
    pub fn distance(&self, to: Point) -> f32 {
        let a = self.x - to.x;
        let b = self.y - to.y;

        a.hypot(b)
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

impl From<Point> for [f32; 2] {
    fn from(point: Point) -> [f32; 2] {
        [point.x, point.y]
    }
}

impl std::ops::Add<f32> for Point {
    type Output = Self;

    fn add(self, rhs: f32) -> Self::Output {
        Self {
            x: self.x + rhs,
            y: self.y + rhs
        }
    }
}

impl std::ops::Sub<f32> for Point {
    type Output = Self;

    fn sub(self, rhs: f32) -> Self::Output {
        Self {
            x: self.x - rhs,
            y: self.y - rhs
        }
    }
}

impl std::ops::Mul<f32> for Point {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self::Output {
        Self {
            x: self.x * rhs,
            y: self.y * rhs
        }
    }
}

impl std::ops::Div<f32> for Point {
    type Output = Self;

    fn div(self, rhs: f32) -> Self::Output {
        Self {
            x: self.x / rhs,
            y: self.y / rhs
        }
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

impl std::ops::Sub<Point> for Point {
    type Output = Vector;

    fn sub(self, point: Point) -> Vector {
        Vector::new(self.x - point.x, self.y - point.y)
    }
}
