//! Draw geometry using meshes of triangles.

/// A set of [`Vertex2D`] and indices representing a list of triangles.
///
/// [`Vertex2D`]: struct.Vertex2D.html
#[derive(Clone, Debug)]
pub struct Mesh2D {
    /// The vertices of the mesh
    pub vertices: Vec<Vertex2D>,

    /// The list of vertex indices that defines the triangles of the mesh.
    ///
    /// Therefore, this list should always have a length that is a multiple of
    /// 3.
    pub indices: Vec<u32>,
}

/// A two-dimensional vertex with some color in __linear__ RGBA.
#[derive(Copy, Clone, Debug)]
#[repr(C)]
pub struct Vertex2D {
    /// The vertex position
    pub position: [f32; 2],
    /// The vertex color in __linear__ RGBA.
    pub color: [f32; 4],
}

#[allow(unsafe_code)]
unsafe impl bytemuck::Zeroable for Vertex2D {}

#[allow(unsafe_code)]
unsafe impl bytemuck::Pod for Vertex2D {}
