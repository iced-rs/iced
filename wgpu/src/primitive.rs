//! Draw using different graphical primitives.
pub mod pipeline;

pub use pipeline::Pipeline;

use crate::graphics::Mesh;

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
