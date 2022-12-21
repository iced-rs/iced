mod compatibility;
mod core;

use crate::program;
use crate::Transformation;
use glow::HasContext;
use iced_graphics::layer;
use iced_native::Rectangle;

#[cfg(feature = "tracing")]
use tracing::info_span;

#[derive(Debug)]
pub enum Pipeline {
    Core(core::Pipeline),
    Compatibility(compatibility::Pipeline),
}

impl Pipeline {
    pub fn new(
        gl: &glow::Context,
        shader_version: &program::Version,
    ) -> Pipeline {
        let gl_version = gl.version();

        // OpenGL 3.0+ and OpenGL ES 3.0+ have instancing (which is what separates `core` from `compatibility`)
        if gl_version.major >= 3 {
            log::info!("Mode: core");
            Pipeline::Core(core::Pipeline::new(gl, shader_version))
        } else {
            log::info!("Mode: compatibility");
            Pipeline::Compatibility(compatibility::Pipeline::new(
                gl,
                shader_version,
            ))
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
        #[cfg(feature = "tracing")]
        let _ = info_span!("Glow::Quad", "DRAW").enter();

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
