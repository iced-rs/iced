//! Control the rotation of some content (like an image) within a space.
use crate::{Degrees, Radians, Size};

/// The strategy used to rotate the content.
///
/// This is used to control the behavior of the layout when the content is rotated
/// by a certain angle.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Rotation {
    /// The element will float while rotating. The layout will be kept exactly as it was
    /// before the rotation.
    ///
    /// This is especially useful when used for animations, as it will avoid the
    /// layout being shifted or resized when smoothly i.e. an icon.
    ///
    /// This is the default.
    Floating(Radians),
    /// The element will be solid while rotating. The layout will be adjusted to fit
    /// the rotated content.
    ///
    /// This allows you to rotate an image and have the layout adjust to fit the new
    /// size of the image.
    Solid(Radians),
}

impl Rotation {
    /// Returns the angle of the [`Rotation`] in [`Radians`].
    pub fn radians(self) -> Radians {
        match self {
            Rotation::Floating(radians) | Rotation::Solid(radians) => radians,
        }
    }

    /// Returns a mutable reference to the angle of the [`Rotation`] in [`Radians`].
    pub fn radians_mut(&mut self) -> &mut Radians {
        match self {
            Rotation::Floating(radians) | Rotation::Solid(radians) => radians,
        }
    }

    /// Returns the angle of the [`Rotation`] in [`Degrees`].
    pub fn degrees(self) -> Degrees {
        Degrees(self.radians().0.to_degrees())
    }

    /// Applies the [`Rotation`] to the given [`Size`], returning
    /// the minimum [`Size`] containing the rotated one.
    pub fn apply(self, size: Size) -> Size {
        match self {
            Self::Floating(_) => size,
            Self::Solid(rotation) => size.rotate(rotation),
        }
    }
}

impl Default for Rotation {
    fn default() -> Self {
        Self::Floating(Radians(0.0))
    }
}

impl From<Radians> for Rotation {
    fn from(radians: Radians) -> Self {
        Self::Floating(radians)
    }
}

impl From<f32> for Rotation {
    fn from(radians: f32) -> Self {
        Self::Floating(Radians(radians))
    }
}
