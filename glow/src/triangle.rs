//! Draw meshes of triangles.
use crate::{settings, Transformation};
use iced_native::{Rectangle, Vector};
use std::mem;

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
        meshes: &[(Vector, Rectangle<u32>, &Mesh2D)],
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

/// A two-dimensional vertex with some color in __linear__ RGBA.
#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct Vertex2D {
    /// The vertex position
    pub position: [f32; 2],
    /// The vertex color in __linear__ RGBA.
    pub color: [f32; 4],
}

/// A set of [`Vertex2D`] and indices representing a list of triangles.
///
/// [`Vertex2D`]: struct.Vertex2D.html
#[derive(Clone, Debug)]
pub struct Mesh2D {
    /// The vertices of the mesh
    pub vertices: Vec<Vertex2D>,
    /// The list of vertex indices that defines the triangles of the mesh.
    ///
    /// Therefore, this list should always have a length that is a multiple of 3.
    pub indices: Vec<u32>,
}
