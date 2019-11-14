use std::f32;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Size {
    /// The width.
    pub width: f32,
    /// The height.
    pub height: f32,
}

impl Size {
    pub const ZERO: Size = Size::new(0., 0.);
    pub const INFINITY: Size = Size::new(f32::INFINITY, f32::INFINITY);

    pub const fn new(width: f32, height: f32) -> Self {
        Size { width, height }
    }

    pub fn pad(&self, padding: f32) -> Self {
        Size {
            width: self.width + padding * 2.0,
            height: self.height + padding * 2.0,
        }
    }
}
