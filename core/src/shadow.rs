use crate::{Color, Vector};

/// A shadow.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Shadow {
    /// The color of the shadow.
    pub color: Color,

    /// The offset of the shadow.
    pub offset: Vector,

    /// The blur radius of the shadow.
    pub blur_radius: f32,
}
