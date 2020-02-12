use iced_native::Point;

use crate::{
    canvas::{Fill, Path, Stroke},
    triangle,
};

#[derive(Debug)]
pub struct Frame {
    width: u32,
    height: u32,
    buffers: lyon::tessellation::VertexBuffers<triangle::Vertex2D, u16>,
}

impl Frame {
    pub(crate) fn new(width: u32, height: u32) -> Frame {
        Frame {
            width,
            height,
            buffers: lyon::tessellation::VertexBuffers::new(),
        }
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn center(&self) -> Point {
        Point::new(self.width as f32 / 2.0, self.height as f32 / 2.0)
    }

    pub fn fill(&mut self, path: Path, fill: Fill) {}

    pub fn stroke(&mut self, path: Path, stroke: Stroke) {}
}
