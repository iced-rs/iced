use crate::{Radians, Vector};

/// An amount of space in 2 dimensions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Size<T = f32> {
    /// The width.
    pub width: T,
    /// The height.
    pub height: T,
}

impl<T> Size<T> {
    /// Creates a new  [`Size`] with the given width and height.
    pub const fn new(width: T, height: T) -> Self {
        Size { width, height }
    }
}

impl Size {
    /// A [`Size`] with zero width and height.
    pub const ZERO: Size = Size::new(0., 0.);

    /// A [`Size`] with a width and height of 1 unit.
    pub const UNIT: Size = Size::new(1., 1.);

    /// A [`Size`] with infinite width and height.
    pub const INFINITY: Size = Size::new(f32::INFINITY, f32::INFINITY);

    /// Returns the minimum of each component of this size and another.
    pub fn min(self, other: Self) -> Self {
        Size {
            width: self.width.min(other.width),
            height: self.height.min(other.height),
        }
    }

    /// Returns the maximum of each component of this size and another.
    pub fn max(self, other: Self) -> Self {
        Size {
            width: self.width.max(other.width),
            height: self.height.max(other.height),
        }
    }

    /// Expands this [`Size`] by the given amount.
    pub fn expand(self, other: impl Into<Size>) -> Self {
        let other = other.into();

        Size {
            width: self.width + other.width,
            height: self.height + other.height,
        }
    }

    /// Rotates the given [`Size`] and returns the minimum [`Size`]
    /// containing it.
    pub fn rotate(self, rotation: Radians) -> Size {
        let radians = f32::from(rotation);

        Size {
            width: (self.width * radians.cos()).abs()
                + (self.height * radians.sin()).abs(),
            height: (self.width * radians.sin()).abs()
                + (self.height * radians.cos()).abs(),
        }
    }
}

impl<T> From<[T; 2]> for Size<T> {
    fn from([width, height]: [T; 2]) -> Self {
        Size { width, height }
    }
}

impl<T> From<(T, T)> for Size<T> {
    fn from((width, height): (T, T)) -> Self {
        Self { width, height }
    }
}

impl<T> From<Vector<T>> for Size<T> {
    fn from(vector: Vector<T>) -> Self {
        Size {
            width: vector.x,
            height: vector.y,
        }
    }
}

impl<T> From<Size<T>> for [T; 2] {
    fn from(size: Size<T>) -> Self {
        [size.width, size.height]
    }
}

impl<T> From<Size<T>> for Vector<T> {
    fn from(size: Size<T>) -> Self {
        Vector::new(size.width, size.height)
    }
}

impl<T> std::ops::Add for Size<T>
where
    T: std::ops::Add<Output = T>,
{
    type Output = Size<T>;

    fn add(self, rhs: Self) -> Self::Output {
        Size {
            width: self.width + rhs.width,
            height: self.height + rhs.height,
        }
    }
}

impl<T> std::ops::Sub for Size<T>
where
    T: std::ops::Sub<Output = T>,
{
    type Output = Size<T>;

    fn sub(self, rhs: Self) -> Self::Output {
        Size {
            width: self.width - rhs.width,
            height: self.height - rhs.height,
        }
    }
}

impl<T> std::ops::Mul<T> for Size<T>
where
    T: std::ops::Mul<Output = T> + Copy,
{
    type Output = Size<T>;

    fn mul(self, rhs: T) -> Self::Output {
        Size {
            width: self.width * rhs,
            height: self.height * rhs,
        }
    }
}

impl<T> std::ops::Mul<Vector<T>> for Size<T>
where
    T: std::ops::Mul<Output = T> + Copy,
{
    type Output = Size<T>;

    fn mul(self, scale: Vector<T>) -> Self::Output {
        Size {
            width: self.width * scale.x,
            height: self.height * scale.y,
        }
    }
}
