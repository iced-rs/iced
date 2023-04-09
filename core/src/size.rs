use crate::{Padding, Vector};
use std::f32;

/// An amount of space in 2 dimensions.
#[derive(Debug, Clone, Copy, PartialEq)]
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

    /// Increments the [`Size`] to account for the given padding.
    pub fn pad(&self, padding: Padding) -> Self {
        Size {
            width: self.width + padding.horizontal() as f32,
            height: self.height + padding.vertical() as f32,
        }
    }
}

impl From<[f32; 2]> for Size {
    fn from([width, height]: [f32; 2]) -> Self {
        Size { width, height }
    }
}

impl From<[u16; 2]> for Size {
    fn from([width, height]: [u16; 2]) -> Self {
        Size::new(width.into(), height.into())
    }
}

impl From<Vector<f32>> for Size {
    fn from(vector: Vector<f32>) -> Self {
        Size {
            width: vector.x,
            height: vector.y,
        }
    }
}

impl From<Size> for [f32; 2] {
    fn from(size: Size) -> [f32; 2] {
        [size.width, size.height]
    }
}

impl From<Size> for Vector<f32> {
    fn from(size: Size) -> Self {
        Vector::new(size.width, size.height)
    }
}
