//! Control the rotation of some content (like an image) with the `RotationLayout` within a
//! space.
use crate::Size;

/// The strategy used to rotate the content.
///
/// This is used to control the behavior of the layout when the content is rotated
/// by a certain angle.
#[derive(Debug, Hash, Clone, Copy, PartialEq, Eq)]
pub enum RotationLayout {
    /// The layout is kept exactly as it was before the rotation.
    ///
    /// This is especially useful when used for animations, as it will avoid the
    /// layout being shifted or resized when smoothly i.e. an icon.
    Keep,
    /// The layout is adjusted to fit the rotated content.
    ///
    /// This allows you to rotate an image and have the layout adjust to fit the new
    /// size of the image.
    Change,
}

impl RotationLayout {
    /// Applies the rotation to the layout while respecting the [`RotationLayout`] strategy.
    /// The rotation is given in radians.
    pub fn apply_to_size(&self, size: Size, rotation: f32) -> Size {
        match self {
            Self::Keep => size,
            Self::Change => Size {
                width: (size.width * rotation.cos()).abs()
                    + (size.height * rotation.sin()).abs(),
                height: (size.width * rotation.sin()).abs()
                    + (size.height * rotation.cos()).abs(),
            },
        }
    }
}
