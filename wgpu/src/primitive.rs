//! Draw using different graphical primitives.
pub mod pipeline;

pub use pipeline::Pipeline;

use crate::core::Rectangle;
use crate::graphics::{Damage, Mesh};

use std::fmt::Debug;

/// The graphical primitives supported by `iced_wgpu`.
pub type Primitive = crate::graphics::Primitive<Custom>;

/// The custom primitives supported by `iced_wgpu`.
#[derive(Debug, Clone, PartialEq)]
pub enum Custom {
    /// A mesh primitive.
    Mesh(Mesh),
    /// A custom pipeline primitive.
    Pipeline(Pipeline),
}

impl Damage for Custom {
    fn bounds(&self) -> Rectangle {
        match self {
            Self::Mesh(mesh) => mesh.bounds(),
            Self::Pipeline(pipeline) => pipeline.bounds,
        }
    }
}
