//! Manage colors for shaders.
use crate::core::Color;

use bytemuck::{Pod, Zeroable};

/// A color packed as 4 floats representing RGBA channels.
#[derive(Debug, Clone, Copy, PartialEq, Zeroable, Pod)]
#[repr(C)]
pub struct Packed([f32; 4]);

impl Packed {
    /// Returns the internal components of the [`Packed`] color.
    pub fn components(self) -> [f32; 4] {
        self.0
    }
}

/// A flag that indicates whether the renderer should perform gamma correction.
pub const GAMMA_CORRECTION: bool = internal::GAMMA_CORRECTION;

/// Packs a [`Color`].
pub fn pack(color: impl Into<Color>) -> Packed {
    Packed(internal::pack(color.into()))
}

#[cfg(not(feature = "web-colors"))]
mod internal {
    use crate::core::Color;

    pub const GAMMA_CORRECTION: bool = true;

    pub fn pack(color: Color) -> [f32; 4] {
        color.into_linear()
    }
}

#[cfg(feature = "web-colors")]
mod internal {
    use crate::core::Color;

    pub const GAMMA_CORRECTION: bool = false;

    pub fn pack(color: Color) -> [f32; 4] {
        [color.r, color.g, color.b, color.a]
    }
}
