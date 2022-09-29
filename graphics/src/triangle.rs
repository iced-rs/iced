//! Draw geometry using meshes of triangles.
use bytemuck::{Pod, Zeroable};

/// A set of [`Vertex2D`] and indices representing a list of triangles.
#[derive(Clone, Debug)]
pub struct Mesh2D {
    /// The vertices of the mesh
    pub vertices: Vec<Vertex2D>,
    /// The list of vertex indices that defines the triangles of the mesh.
    pub indices: Vec<u32>,
}

/// A two-dimensional vertex.
#[derive(Copy, Clone, Debug, Zeroable, Pod)]
#[repr(C)]
pub struct Vertex2D {
    /// The vertex position in 2D space.
    pub position: [f32; 2],
}

/// Convert from lyon's position data to Iced's Vertex2D type.
impl Vertex2D {
    /// Converts from [`lyon::math::Point`] to [`Vertex2D`]. Used for generating primitives.
    pub fn from(points: Vec<lyon::math::Point>) -> Vec<Vertex2D> {
        points.iter().map(|p| Vertex2D { position: [p.x, p.y]}).collect()
    }
}
