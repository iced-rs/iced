use crate::Transformation;

#[derive(Debug)]
pub struct Viewport {
    width: u32,
    height: u32,
    scale_factor: f32,
    transformation: Transformation,
}

impl Viewport {
    pub fn new(width: u32, height: u32, scale_factor: f64) -> Viewport {
        Viewport {
            width,
            height,
            scale_factor: scale_factor as f32,
            transformation: Transformation::orthographic(width, height),
        }
    }

    pub fn dimensions(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    pub fn scale_factor(&self) -> f32 {
        self.scale_factor
    }

    pub(crate) fn transformation(&self) -> Transformation {
        self.transformation
    }
}
