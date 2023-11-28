use crate::core::Rectangle;
use crate::primitive::pipeline::Primitive;

use std::sync::Arc;

#[derive(Clone, Debug)]
/// A custom primitive which can be used to render primitives associated with a custom pipeline.
pub struct Pipeline {
    /// The bounds of the [`Pipeline`].
    pub bounds: Rectangle,

    /// The viewport of the [`Pipeline`].
    pub viewport: Rectangle,

    /// The [`Primitive`] to render.
    pub primitive: Arc<dyn Primitive>,
}
