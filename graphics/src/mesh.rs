//! Draw triangles!
use crate::color;
use crate::core::{Rectangle, Transformation};
use crate::gradient;

use bytemuck::{Pod, Zeroable};

use std::sync::atomic::{self, AtomicU64};
use std::sync::{Arc, Weak};

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
            Self::Solid { transformation, .. } | Self::Gradient { transformation, .. } => {
                *transformation
            }
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

/// A cache of multiple meshes.
#[derive(Debug, Clone)]
pub struct Cache {
    id: Id,
    batch: Arc<[Mesh]>,
    version: usize,
}

/// The unique id of a [`Cache`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Id(u64);

impl Cache {
    /// Creates a new [`Cache`] for the given meshes.
    pub fn new(meshes: Arc<[Mesh]>) -> Self {
        static NEXT_ID: AtomicU64 = AtomicU64::new(0);

        Self {
            id: Id(NEXT_ID.fetch_add(1, atomic::Ordering::Relaxed)),
            batch: meshes,
            version: 0,
        }
    }

    /// Returns the [`Id`] of the [`Cache`].
    pub fn id(&self) -> Id {
        self.id
    }

    /// Returns the current version of the [`Cache`].
    pub fn version(&self) -> usize {
        self.version
    }

    /// Returns the batch of meshes in the [`Cache`].
    pub fn batch(&self) -> &[Mesh] {
        &self.batch
    }

    /// Returns a [`Weak`] reference to the contents of the [`Cache`].
    pub fn downgrade(&self) -> Weak<[Mesh]> {
        Arc::downgrade(&self.batch)
    }

    /// Returns true if the [`Cache`] is empty.
    pub fn is_empty(&self) -> bool {
        self.batch.is_empty()
    }

    /// Updates the [`Cache`] with the given meshes.
    pub fn update(&mut self, meshes: Arc<[Mesh]>) {
        self.batch = meshes;
        self.version += 1;
    }
}

/// A renderer capable of drawing a [`Mesh`].
pub trait Renderer {
    /// Draws the given [`Mesh`].
    fn draw_mesh(&mut self, mesh: Mesh);

    /// Draws the given [`Cache`].
    fn draw_mesh_cache(&mut self, cache: Cache);
}
