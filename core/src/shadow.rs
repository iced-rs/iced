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

    /// The spread radius of the shadow.
    ///
    /// Positive values expand the shadow outward (larger than the element).
    /// Negative values contract the shadow inward (smaller than the element).
    pub spread_radius: f32,

    /// Whether the shadow is inset (inside the element) or outset (outside).
    /// Default is `false` (outset shadow).
    pub inset: bool,
}

impl Shadow {
    /// Creates a new outset (default) shadow.
    pub fn new(color: Color, offset: Vector, blur_radius: f32) -> Self {
        Self {
            color,
            offset,
            blur_radius,
            spread_radius: 0.0,
            inset: false,
        }
    }

    /// Creates a new inset shadow.
    pub fn inset(color: Color, offset: Vector, blur_radius: f32) -> Self {
        Self {
            color,
            offset,
            blur_radius,
            spread_radius: 0.0,
            inset: true,
        }
    }

    /// Sets the spread radius of the shadow.
    pub fn with_spread(mut self, spread_radius: f32) -> Self {
        self.spread_radius = spread_radius;
        self
    }

    /// Sets whether the shadow is inset.
    pub fn with_inset(mut self, inset: bool) -> Self {
        self.inset = inset;
        self
    }
}
