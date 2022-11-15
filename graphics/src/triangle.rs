//! Draw geometry using meshes of triangles.
use crate::Color;
#[cfg(not(target_arch = "wasm32"))]
use crate::Gradient;

use bytemuck::{Pod, Zeroable};

/// A set of [`Vertex2D`] and indices representing a list of triangles.
#[derive(Clone, Debug)]
pub struct Mesh2D {
    /// The vertices of the mesh
    pub vertices: Vec<Vertex2D>,
    /// The list of vertex indices that defines the triangles of the mesh.
    ///
    /// Therefore, this list should always have a length that is a multiple of 3.
    pub indices: Vec<u32>,
}

/// A two-dimensional vertex.
#[derive(Copy, Clone, Debug, Zeroable, Pod)]
#[repr(C)]
pub struct Vertex2D {
    /// The vertex position in 2D space.
    pub position: [f32; 2],
}

#[derive(Debug, Clone, PartialEq)]
/// Supported shaders for triangle primitives.
pub enum Style {
    /// Fill a primitive with a solid color.
    Solid(Color),
    #[cfg(not(target_arch = "wasm32"))]
    /// Fill a primitive with an interpolated color.
    Gradient(Gradient),
}

impl From<Color> for Style {
    fn from(color: Color) -> Self {
        Self::Solid(color)
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl From<Gradient> for Style {
    fn from(gradient: Gradient) -> Self {
        Self::Gradient(gradient)
    }
}
