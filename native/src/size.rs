use std::f32;

/// An amount of space in 2 dimensions.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Size {
    /// The width.
    pub width: f32,
    /// The height.
    pub height: f32,
}

impl Size {
    /// A [`Size`] with zero width and height.
    ///
    /// [`Size`]: struct.Size.html
    pub const ZERO: Size = Size::new(0., 0.);

    /// A [`Size`] with infinite width and height.
    ///
    /// [`Size`]: struct.Size.html
    pub const INFINITY: Size = Size::new(f32::INFINITY, f32::INFINITY);

    /// A [`Size`] of infinite width and height.
    ///
    /// [`Size`]: struct.Size.html
    pub const fn new(width: f32, height: f32) -> Self {
        Size { width, height }
    }

    /// Increments the [`Size`] to account for the given padding.
    ///
    /// [`Size`]: struct.Size.html
    pub fn pad(&self, padding: f32) -> Self {
        Size {
            width: self.width + padding * 2.0,
            height: self.height + padding * 2.0,
        }
    }
}
