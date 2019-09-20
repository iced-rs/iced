/// A 2D vector.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vector {
    pub x: f32,
    pub y: f32,
}

impl Vector {
    /// Creates a new [`Vector`] with the given components.
    ///
    /// [`Vector`]: struct.Vector.html
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}
