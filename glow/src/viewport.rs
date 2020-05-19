use crate::Transformation;

/// A viewing region for displaying computer graphics.
#[derive(Debug)]
pub struct Viewport {
    width: u32,
    height: u32,
    transformation: Transformation,
}

impl Viewport {
    /// Creates a new [`Viewport`] with the given dimensions.
    pub fn new(width: u32, height: u32) -> Viewport {
        Viewport {
            width,
            height,
            transformation: Transformation::orthographic(width, height),
        }
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    /// Returns the dimensions of the [`Viewport`].
    pub fn dimensions(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    pub(crate) fn transformation(&self) -> Transformation {
        self.transformation
    }
}
