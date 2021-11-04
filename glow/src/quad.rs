mod compatibility;
mod core;

use crate::Transformation;
use glow::HasContext;
use iced_graphics::layer;
use iced_native::Rectangle;

#[derive(Debug)]
pub enum Pipeline {
    Core(core::Pipeline),
    Compatibility(compatibility::Pipeline),
}

impl Pipeline {
    pub fn new(gl: &glow::Context) -> Pipeline {
        let version = gl.version();
        if version.is_embedded || version.major == 2 {
            Pipeline::Compatibility(compatibility::Pipeline::new(gl))
        } else {
            Pipeline::Core(core::Pipeline::new(gl))
        }
    }

    pub fn draw(
        &mut self,
        gl: &glow::Context,
        target_height: u32,
        instances: &[layer::Quad],
        transformation: Transformation,
        scale: f32,
        bounds: Rectangle<u32>,
    ) {
        match self {
            Pipeline::Core(pipeline) => {
                pipeline.draw(
                    gl,
                    target_height,
                    instances,
                    transformation,
                    scale,
                    bounds,
                );
            }
            Pipeline::Compatibility(pipeline) => {
                pipeline.draw(
                    gl,
                    target_height,
                    instances,
                    transformation,
                    scale,
                    bounds,
                );
            }
        }
    }
}
