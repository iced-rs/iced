use crate::core::{Rectangle, Size};

use std::fmt::{Debug, Formatter};
use std::sync::Arc;

/// Data of a screenshot, captured with `window::screenshot()`.
///
/// The `bytes` of this screenshot will always be ordered as `RGBA` in the sRGB color space.
#[derive(Clone)]
pub struct Screenshot {
    /// The bytes of the [`Screenshot`].
    pub bytes: Arc<Vec<u8>>,
    /// The size of the [`Screenshot`].
    pub size: Size<u32>,
}

impl Debug for Screenshot {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Screenshot: {{ \n bytes: {}\n size: {:?} }}",
            self.bytes.len(),
            self.size
        )
    }
}

impl Screenshot {
    /// Creates a new [`Screenshot`].
    pub fn new(bytes: Vec<u8>, size: Size<u32>) -> Self {
        Self {
            bytes: Arc::new(bytes),
            size,
        }
    }

    /// Crops a [`Screenshot`] to the provided `region`. This will always be relative to the
    /// top-left corner of the [`Screenshot`].
    pub fn crop(&self, region: Rectangle<u32>) -> Result<Self, CropError> {
        if region.width == 0 || region.height == 0 {
            return Err(CropError::Zero);
        }

        if region.x + region.width > self.size.width
            || region.y + region.height > self.size.height
        {
            return Err(CropError::OutOfBounds);
        }

        // Image is always RGBA8 = 4 bytes per pixel
        const PIXEL_SIZE: usize = 4;

        let bytes_per_row = self.size.width as usize * PIXEL_SIZE;
        let row_range = region.y as usize..(region.y + region.height) as usize;
        let column_range = region.x as usize * PIXEL_SIZE
            ..(region.x + region.width) as usize * PIXEL_SIZE;

        let chopped = self.bytes.chunks(bytes_per_row).enumerate().fold(
            vec![],
            |mut acc, (row, bytes)| {
                if row_range.contains(&row) {
                    acc.extend(&bytes[column_range.clone()]);
                }

                acc
            },
        );

        Ok(Self {
            bytes: Arc::new(chopped),
            size: Size::new(region.width, region.height),
        })
    }
}

impl AsRef<[u8]> for Screenshot {
    fn as_ref(&self) -> &[u8] {
        &self.bytes
    }
}

#[derive(Debug, thiserror::Error)]
/// Errors that can occur when cropping a [`Screenshot`].
pub enum CropError {
    #[error("The cropped region is out of bounds.")]
    /// The cropped region's size is out of bounds.
    OutOfBounds,
    #[error("The cropped region is not visible.")]
    /// The cropped region's size is zero.
    Zero,
}
