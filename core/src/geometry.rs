/// A two-dimensional vertex which has a color
#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct Vertex2D {
    /// The vertex position
    pub position: [f32; 2],
    /// The vertex color in rgba
    pub color: [f32; 4],
}

/// A set of [`Vertex2D`] and indices for drawing some 2D geometry on the GPU.
///
/// [`Vertex2D`]: struct.Vertex2D.html
#[derive(Clone, Debug)]
pub struct Geometry2D {
    /// The vertices for this geometry
    pub vertices: Vec<Vertex2D>,
    /// The indices for this geometry
    pub indices: Vec<u16>,
}
