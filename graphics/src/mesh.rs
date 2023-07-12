//! Draw triangles!
use crate::color;
use crate::core::{Rectangle, Size};
use crate::gradient;
use crate::Damage;

use bytemuck::{Pod, Zeroable};

/// A low-level primitive to render a mesh of triangles.
#[derive(Debug, Clone, PartialEq)]
pub enum Mesh {
    /// A mesh with a solid color.
    Solid {
        /// The vertices and indices of the mesh.
        buffers: Indexed<SolidVertex2D>,

        /// The size of the drawable region of the mesh.
        ///
        /// Any geometry that falls out of this region will be clipped.
        size: Size,
    },
    /// A mesh with a gradient.
    Gradient {
        /// The vertices and indices of the mesh.
        buffers: Indexed<GradientVertex2D>,

        /// The size of the drawable region of the mesh.
        ///
        /// Any geometry that falls out of this region will be clipped.
        size: Size,
    },
}

impl Damage for Mesh {
    fn bounds(&self) -> Rectangle {
        match self {
            Self::Solid { size, .. } | Self::Gradient { size, .. } => {
                Rectangle::with_size(*size)
            }
        }
    }
}

/// A set of [`Vertex2D`] and indices representing a list of triangles.
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
