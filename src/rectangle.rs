use crate::Point;

/// A rectangle.
#[derive(Debug, PartialEq, Copy, Clone)]
pub struct Rectangle {
    /// X coordinate of the top-left corner.
    pub x: f32,

    /// Y coordinate of the top-left corner.
    pub y: f32,

    /// Width of the rectangle.
    pub width: f32,

    /// Height of the rectangle.
    pub height: f32,
}

impl Rectangle {
    /// Returns true if the given [`Point`] is contained in the [`Rectangle`].
    ///
    /// [`Point`]: struct.Point.html
    /// [`Rectangle`]: struct.Rectangle.html
    pub fn contains(&self, point: Point) -> bool {
        self.x <= point.x
            && point.x <= self.x + self.width
            && self.y <= point.y
            && point.y <= self.y + self.height
    }
}
