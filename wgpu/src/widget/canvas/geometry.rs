use crate::Primitive;
use std::sync::Arc;

#[derive(Debug)]
pub struct Geometry(Arc<Primitive>);

impl Geometry {
    pub(crate) fn from_primitive(primitive: Arc<Primitive>) -> Self {
        Self(primitive)
    }

    pub(crate) fn into_primitive(self) -> Arc<Primitive> {
        self.0
    }
}
