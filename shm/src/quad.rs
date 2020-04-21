use crate::Transformation;
use iced_native::Rectangle;

//use zerocopy::AsBytes;

#[derive(Debug)]
pub struct Pipeline {}

impl Pipeline {
    pub fn new(_device: &()) -> Pipeline {
        Self {}
    }

    pub fn draw(
        &mut self,
        _instances: &[Quad],
        _transformation: Transformation,
        _scale: f32,
        _bounds: Rectangle<u32>,
        _target: &(),
    ) {
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Quad {
    pub position: [f32; 2],
    pub scale: [f32; 2],
    pub color: [f32; 4],
    pub border_color: [f32; 4],
    pub border_radius: f32,
    pub border_width: f32,
}
