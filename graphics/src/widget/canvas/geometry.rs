use crate::Primitive;

/// A bunch of shapes that can be drawn.
///
/// [`Geometry`] can be easily generated with a [`Frame`] or stored in a
/// [`Cache`].
///
/// [`Frame`]: crate::widget::canvas::Frame
/// [`Cache`]: crate::widget::canvas::Cache
#[derive(Debug, Clone)]
pub struct Geometry(Primitive);

impl Geometry {
    pub(crate) fn from_primitive(primitive: Primitive) -> Self {
        Self(primitive)
    }

    /// Turns the [`Geometry`] into a [`Primitive`].
    ///
    /// This can be useful if you are building a custom widget.
    pub fn into_primitive(self) -> Primitive {
        self.0
    }
}

impl From<Geometry> for Primitive {
    fn from(geometry: Geometry) -> Primitive {
        geometry.0
    }
}
