//! Draw using different graphical primitives.
use crate::core::Rectangle;
use crate::custom;
use crate::graphics::{Damage, Mesh};
use std::any::Any;
use std::fmt::Debug;
use std::sync::Arc;

/// The graphical primitives supported by `iced_wgpu`.
pub type Primitive = crate::graphics::Primitive<Custom>;

/// The custom primitives supported by `iced_wgpu`.
#[derive(Debug, Clone, PartialEq)]
pub enum Custom {
    /// A mesh primitive.
    Mesh(Mesh),
    /// A custom shader primitive
    Shader(Shader),
}

impl Custom {
    /// Create a custom [`Shader`] primitive.
    pub fn shader<P: custom::Primitive>(
        bounds: Rectangle,
        primitive: P,
    ) -> Self {
        Self::Shader(Shader {
            bounds,
            primitive: Arc::new(primitive),
        })
    }
}

impl Damage for Custom {
    fn bounds(&self) -> Rectangle {
        match self {
            Self::Mesh(mesh) => mesh.bounds(),
            Self::Shader(shader) => shader.bounds,
        }
    }
}

#[derive(Clone, Debug)]
/// A custom primitive which can be used to render primitives associated with a custom pipeline.
pub struct Shader {
    /// The bounds of the [`Shader`].
    pub bounds: Rectangle,

    /// The [`custom::Primitive`] to render.
    pub primitive: Arc<dyn custom::Primitive>,
}

impl PartialEq for Shader {
    fn eq(&self, other: &Self) -> bool {
        self.primitive.type_id() == other.primitive.type_id()
    }
}
