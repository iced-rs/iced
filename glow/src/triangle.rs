//! Draw meshes of triangles.
use crate::{settings, Transformation};
use iced_graphics::layer;

pub use iced_graphics::triangle::Mesh2D;

const UNIFORM_BUFFER_SIZE: usize = 100;
const VERTEX_BUFFER_SIZE: usize = 10_000;
const INDEX_BUFFER_SIZE: usize = 10_000;

#[derive(Debug)]
pub(crate) struct Pipeline {}

impl Pipeline {
    pub fn new(
        gl: &glow::Context,
        antialiasing: Option<settings::Antialiasing>,
    ) -> Pipeline {
        Pipeline {}
    }

    pub fn draw(
        &mut self,
        gl: &glow::Context,
        target_width: u32,
        target_height: u32,
        transformation: Transformation,
        scale_factor: f32,
        meshes: &[layer::Mesh<'_>],
    ) {
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct Uniforms {
    transform: [f32; 16],
    // We need to align this to 256 bytes to please `wgpu`...
    // TODO: Be smarter and stop wasting memory!
    _padding_a: [f32; 32],
    _padding_b: [f32; 16],
}

impl Default for Uniforms {
    fn default() -> Self {
        Self {
            transform: *Transformation::identity().as_ref(),
            _padding_a: [0.0; 32],
            _padding_b: [0.0; 16],
        }
    }
}

impl From<Transformation> for Uniforms {
    fn from(transformation: Transformation) -> Uniforms {
        Self {
            transform: transformation.into(),
            _padding_a: [0.0; 32],
            _padding_b: [0.0; 16],
        }
    }
}
