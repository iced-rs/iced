use crate::Point;

/// A generic rectangle.
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub struct Rectangle<T> {
    /// X coordinate of the top-left corner.
    pub x: T,

    /// Y coordinate of the top-left corner.
    pub y: T,

    /// Width of the rectangle.
    pub width: T,

    /// Height of the rectangle.
    pub height: T,
}

impl Rectangle<f32> {
    /// Returns true if the given [`Point`] is contained in the [`Rectangle`].
    ///
    /// [`Point`]: type.Point.html
    /// [`Rectangle`]: struct.Rectangle.html
    pub fn contains(&self, point: Point) -> bool {
        self.x <= point.x
            && point.x <= self.x + self.width
            && self.y <= point.y
            && point.y <= self.y + self.height
    }
}
