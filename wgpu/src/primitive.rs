use crate::core::Rectangle;
use crate::graphics::{Damage, Mesh};

pub type Primitive = crate::graphics::Primitive<Custom>;

#[derive(Debug, Clone, PartialEq)]
pub enum Custom {
    Mesh(Mesh),
}

impl Damage for Custom {
    fn bounds(&self) -> Rectangle {
        match self {
            Self::Mesh(mesh) => mesh.bounds(),
        }
    }
}
