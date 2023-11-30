use crate::Vector;

use num_traits::{Float, Num};
use std::fmt;

/// A 2D point.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Point<T = f32> {
    /// The X coordinate.
    pub x: T,

    /// The Y coordinate.
    pub y: T,
}

impl Point {
    /// The origin (i.e. a [`Point`] at (0, 0)).
    pub const ORIGIN: Self = Self::new(0.0, 0.0);
}

impl<T: Num> Point<T> {
    /// Creates a new [`Point`] with the given coordinates.
    pub const fn new(x: T, y: T) -> Self {
        Self { x, y }
    }

    /// Computes the distance to another [`Point`].
    pub fn distance(&self, to: Self) -> T
    where
        T: Float,
    {
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

impl From<[u16; 2]> for Point<u16> {
    fn from([x, y]: [u16; 2]) -> Self {
        Point::new(x, y)
    }
}

impl From<Point> for [f32; 2] {
    fn from(point: Point) -> [f32; 2] {
        [point.x, point.y]
    }
}

impl<T> std::ops::Add<Vector<T>> for Point<T>
where
    T: std::ops::Add<Output = T>,
{
    type Output = Self;

    fn add(self, vector: Vector<T>) -> Self {
        Self {
            x: self.x + vector.x,
            y: self.y + vector.y,
        }
    }
}

impl<T> std::ops::Sub<Vector<T>> for Point<T>
where
    T: std::ops::Sub<Output = T>,
{
    type Output = Self;

    fn sub(self, vector: Vector<T>) -> Self {
        Self {
            x: self.x - vector.x,
            y: self.y - vector.y,
        }
    }
}

impl<T> std::ops::Sub<Point<T>> for Point<T>
where
    T: std::ops::Sub<Output = T>,
{
    type Output = Vector<T>;

    fn sub(self, point: Self) -> Vector<T> {
        Vector::new(self.x - point.x, self.y - point.y)
    }
}

impl<T> fmt::Display for Point<T>
where
    T: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Point {{ x: {}, y: {} }}", self.x, self.y)
    }
}
