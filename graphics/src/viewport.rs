use crate::{Size, Transformation};

/// A viewing region for displaying computer graphics.
#[derive(Debug)]
pub struct Viewport {
    physical_size: Size<u32>,
    logical_size: Size<f32>,
    scale_factor: f64,
    projection: Transformation,
}

impl Viewport {
    /// Creates a new [`Viewport`] with the given physical dimensions and scale
    /// factor.
    ///
    /// [`Viewport`]: struct.Viewport.html
    pub fn with_physical_size(size: Size<u32>, scale_factor: f64) -> Viewport {
        Viewport {
            physical_size: size,
            logical_size: Size::new(
                (size.width as f64 / scale_factor) as f32,
                (size.height as f64 / scale_factor) as f32,
            ),
            scale_factor,
            projection: Transformation::orthographic(size.width, size.height),
        }
    }

    pub fn physical_size(&self) -> Size<u32> {
        self.physical_size
    }

    pub fn physical_height(&self) -> u32 {
        self.physical_size.height
    }

    pub fn logical_size(&self) -> Size<f32> {
        self.logical_size
    }

    pub fn scale_factor(&self) -> f64 {
        self.scale_factor
    }

    pub fn projection(&self) -> Transformation {
        self.projection
    }
}
