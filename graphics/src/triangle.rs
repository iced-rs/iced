//! Draw geometry using meshes of triangles.
use bytemuck::{Pod, Zeroable};

/// A set of [`Vertex2D`] and indices representing a list of triangles.
#[derive(Clone, Debug)]
pub struct Mesh2D<T> {
    /// The vertices of the mesh
    pub vertices: Vec<T>,

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

/// A two-dimensional vertex with a color.
#[derive(Copy, Clone, Debug, Zeroable, Pod)]
#[repr(C)]
pub struct ColoredVertex2D {
    /// The vertex position in 2D space.
    pub position: [f32; 2],

    /// The color of the vertex in __linear__ RGBA.
    pub color: [f32; 4],
}
