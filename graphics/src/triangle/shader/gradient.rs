//! TODO
use bytemuck::{Pod, Zeroable};
use iced_native::{Color, Point};

#[derive(Debug, Clone)]
/// TODO
pub enum Gradient {
    /// TODO
    Linear {
        /// TODO
        start: Point,
        /// TODO
        end: Point,
        /// TODO
        stops: Vec<ColorStop>,
    },
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Zeroable, Pod)]
/// TODO
pub struct ColorStop {
    offset: f32,
    _padding: [u32; 3],
    color: [f32; 4],
}

impl From<(f32, Color)> for ColorStop {
    fn from((offset, color): (f32, Color)) -> Self {
        Self {
            offset,
            _padding: [0; 3],
            color: color.into_linear(),
        }
    }
}
