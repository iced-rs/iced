//! A collection of triangle primitives.
use crate::triangle;
use crate::{Point, Rectangle};

/// A mesh of triangles.
#[derive(Debug, Clone, Copy)]
pub struct Mesh<'a> {
    /// The origin of the vertices of the [`Mesh`].
    pub origin: Point,

    /// The vertex and index buffers of the [`Mesh`].
    pub buffers: &'a triangle::Mesh2D,

    /// The clipping bounds of the [`Mesh`].
    pub clip_bounds: Rectangle<f32>,

    /// The shader of the [`Mesh`].
    pub style: &'a triangle::Style,
}

/// Returns the number of total vertices & total indices of all [`Mesh`]es.
pub fn attribute_count_of<'a>(meshes: &'a [Mesh<'a>]) -> (usize, usize) {
    meshes
        .iter()
        .map(|Mesh { buffers, .. }| {
            (buffers.vertices.len(), buffers.indices.len())
        })
        .fold((0, 0), |(total_v, total_i), (v, i)| {
            (total_v + v, total_i + i)
        })
}
