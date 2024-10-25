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

impl<T> From<[T; 2]> for Point<T>
where
    T: Num,
{
    fn from([x, y]: [T; 2]) -> Self {
        Point { x, y }
    }
}

impl<T> From<(T, T)> for Point<T>
where
    T: Num,
{
    fn from((x, y): (T, T)) -> Self {
        Self { x, y }
    }
}

impl<T> From<Point<T>> for [T; 2] {
    fn from(point: Point<T>) -> [T; 2] {
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

impl<T> std::ops::AddAssign<Vector<T>> for Point<T>
where
    T: std::ops::AddAssign,
{
    fn add_assign(&mut self, vector: Vector<T>) {
        self.x += vector.x;
        self.y += vector.y;
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

impl<T> std::ops::SubAssign<Vector<T>> for Point<T>
where
    T: std::ops::SubAssign,
{
    fn sub_assign(&mut self, vector: Vector<T>) {
        self.x -= vector.x;
        self.y -= vector.y;
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

impl<T> std::ops::SubAssign<Point<T>> for Point<T>
where
    T: std::ops::SubAssign,
{
    fn sub_assign(&mut self, point: Point<T>) {
        self.x -= point.x;
        self.y -= point.y;
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
