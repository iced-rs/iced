//! Control the rotation of some content (like an image) within a space.
use crate::{Radians, Size};

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

    /// Rotates the given [`Size`].
    pub fn apply(self, size: Size) -> Size {
        match self {
            Self::Floating(_) => size,
            Self::Solid(rotation) => {
                let radians = f32::from(rotation);

                Size {
                    width: (size.width * radians.cos()).abs()
                        + (size.height * radians.sin()).abs(),
                    height: (size.width * radians.sin()).abs()
                        + (size.height * radians.cos()).abs(),
                }
            }
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
