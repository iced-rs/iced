use crate::Primitive;

#[derive(Debug, Clone)]
pub struct Geometry(Primitive);

impl Geometry {
    pub(crate) fn from_primitive(primitive: Primitive) -> Self {
        Self(primitive)
    }

    pub fn into_primitive(self) -> Primitive {
        self.0
    }
}

impl From<Geometry> for Primitive {
    fn from(geometry: Geometry) -> Primitive {
        geometry.0
    }
}
