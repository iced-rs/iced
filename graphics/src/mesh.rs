//! Draw triangles!
use crate::color;
use crate::core::{Rectangle, Transformation};
use crate::gradient;

use bytemuck::{Pod, Zeroable};

/// A low-level primitive to render a mesh of triangles.
#[derive(Debug, Clone, PartialEq)]
pub enum Mesh {
    /// A mesh with a solid color.
    Solid {
        /// The vertices and indices of the mesh.
        buffers: Indexed<SolidVertex2D>,

        /// The [`Transformation`] for the vertices of the [`Mesh`].
        transformation: Transformation,

        /// The clip bounds of the [`Mesh`].
        clip_bounds: Rectangle,
    },
    /// A mesh with a gradient.
    Gradient {
        /// The vertices and indices of the mesh.
        buffers: Indexed<GradientVertex2D>,

        /// The [`Transformation`] for the vertices of the [`Mesh`].
        transformation: Transformation,

        /// The clip bounds of the [`Mesh`].
        clip_bounds: Rectangle,
    },
}

impl Mesh {
    /// Returns the indices of the [`Mesh`].
    pub fn indices(&self) -> &[u32] {
        match self {
            Self::Solid { buffers, .. } => &buffers.indices,
            Self::Gradient { buffers, .. } => &buffers.indices,
        }
    }

    /// Returns the [`Transformation`] of the [`Mesh`].
    pub fn transformation(&self) -> Transformation {
        match self {
            Self::Solid { transformation, .. }
            | Self::Gradient { transformation, .. } => *transformation,
        }
    }

    /// Returns the clip bounds of the [`Mesh`].
    pub fn clip_bounds(&self) -> Rectangle {
        match self {
            Self::Solid {
                clip_bounds,
                transformation,
                ..
            }
            | Self::Gradient {
                clip_bounds,
                transformation,
                ..
            } => *clip_bounds * *transformation,
        }
    }
}

/// A set of vertices and indices representing a list of triangles.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Indexed<T> {
    /// The vertices of the mesh
    pub vertices: Vec<T>,

    /// The list of vertex indices that defines the triangles of the mesh.
    ///
    /// Therefore, this list should always have a length that is a multiple of 3.
    pub indices: Vec<u32>,
}

/// A two-dimensional vertex with a color.
#[derive(Copy, Clone, Debug, PartialEq, Zeroable, Pod)]
#[repr(C)]
pub struct SolidVertex2D {
    /// The vertex position in 2D space.
    pub position: [f32; 2],

    /// The color of the vertex in __linear__ RGBA.
    pub color: color::Packed,
}

/// A vertex which contains 2D position & packed gradient data.
#[derive(Copy, Clone, Debug, PartialEq, Zeroable, Pod)]
#[repr(C)]
pub struct GradientVertex2D {
    /// The vertex position in 2D space.
    pub position: [f32; 2],

    /// The packed vertex data of the gradient.
    pub gradient: gradient::Packed,
}

/// The result of counting the attributes of a set of meshes.
#[derive(Debug, Clone, Copy, Default)]
pub struct AttributeCount {
    /// The total amount of solid vertices.
    pub solid_vertices: usize,

    /// The total amount of solid meshes.
    pub solids: usize,

    /// The total amount of gradient vertices.
    pub gradient_vertices: usize,

    /// The total amount of gradient meshes.
    pub gradients: usize,

    /// The total amount of indices.
    pub indices: usize,
}

/// Returns the number of total vertices & total indices of all [`Mesh`]es.
pub fn attribute_count_of(meshes: &[Mesh]) -> AttributeCount {
    meshes
        .iter()
        .fold(AttributeCount::default(), |mut count, mesh| {
            match mesh {
                Mesh::Solid { buffers, .. } => {
                    count.solids += 1;
                    count.solid_vertices += buffers.vertices.len();
                    count.indices += buffers.indices.len();
                }
                Mesh::Gradient { buffers, .. } => {
                    count.gradients += 1;
                    count.gradient_vertices += buffers.vertices.len();
                    count.indices += buffers.indices.len();
                }
            }

            count
        })
}

/// A renderer capable of drawing a [`Mesh`].
pub trait Renderer {
    /// Draws the given [`Mesh`].
    fn draw_mesh(&mut self, mesh: Mesh);
}
