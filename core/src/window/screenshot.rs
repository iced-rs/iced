//! Take screenshots of a window.
use crate::{Bytes, Rectangle, Size};

use std::fmt::{Debug, Formatter};

/// Data of a screenshot, captured with `window::screenshot()`.
///
/// The `bytes` of this screenshot will always be ordered as `RGBA` in the `sRGB` color space.
#[derive(Clone)]
pub struct Screenshot {
    /// The RGBA bytes of the [`Screenshot`].
    pub rgba: Bytes,
    /// The size of the [`Screenshot`] in physical pixels.
    pub size: Size<u32>,
    /// The scale factor of the [`Screenshot`]. This can be useful when converting between widget
    /// bounds (which are in logical pixels) to crop screenshots.
    pub scale_factor: f32,
}

impl Debug for Screenshot {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Screenshot: {{ \n bytes: {}\n scale: {}\n size: {:?} }}",
            self.rgba.len(),
            self.scale_factor,
            self.size
        )
    }
}

impl Screenshot {
    /// Creates a new [`Screenshot`].
    pub fn new(
        rgba: impl Into<Bytes>,
        size: Size<u32>,
        scale_factor: f32,
    ) -> Self {
        Self {
            rgba: rgba.into(),
            size,
            scale_factor,
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

        let chopped = self.rgba.chunks(bytes_per_row).enumerate().fold(
            vec![],
            |mut acc, (row, bytes)| {
                if row_range.contains(&row) {
                    acc.extend(&bytes[column_range.clone()]);
                }

                acc
            },
        );

        Ok(Self {
            rgba: Bytes::from(chopped),
            size: Size::new(region.width, region.height),
            scale_factor: self.scale_factor,
        })
    }
}

impl AsRef<[u8]> for Screenshot {
    fn as_ref(&self) -> &[u8] {
        &self.rgba
    }
}

impl From<Screenshot> for Bytes {
    fn from(screenshot: Screenshot) -> Self {
        screenshot.rgba
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
