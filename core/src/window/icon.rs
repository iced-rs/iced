//! Change the icon of a window.
use crate::Size;

use std::mem;

/// Builds an  [`Icon`] from its RGBA pixels in the `sRGB` color space.
pub fn from_rgba(
    rgba: Vec<u8>,
    width: u32,
    height: u32,
) -> Result<Icon, Error> {
    const PIXEL_SIZE: usize = mem::size_of::<u8>() * 4;

    if rgba.len() % PIXEL_SIZE != 0 {
        return Err(Error::ByteCountNotDivisibleBy4 {
            byte_count: rgba.len(),
        });
    }

    let pixel_count = rgba.len() / PIXEL_SIZE;

    if pixel_count != (width * height) as usize {
        return Err(Error::DimensionsVsPixelCount {
            width,
            height,
            width_x_height: (width * height) as usize,
            pixel_count,
        });
    }

    Ok(Icon {
        rgba,
        size: Size::new(width, height),
    })
}

/// An window icon normally used for the titlebar or taskbar.
#[derive(Debug, Clone)]
pub struct Icon {
    rgba: Vec<u8>,
    size: Size<u32>,
}

impl Icon {
    /// Returns the raw data of the [`Icon`].
    pub fn into_raw(self) -> (Vec<u8>, Size<u32>) {
        (self.rgba, self.size)
    }
}

#[derive(Debug, thiserror::Error)]
/// An error produced when using [`from_rgba`] with invalid arguments.
pub enum Error {
    /// Produced when the length of the `rgba` argument isn't divisible by 4, thus `rgba` can't be
    /// safely interpreted as 32bpp RGBA pixels.
    #[error(
        "The provided RGBA data (with length {byte_count}) isn't divisible \
        by 4. Therefore, it cannot be safely interpreted as 32bpp RGBA pixels"
    )]
    ByteCountNotDivisibleBy4 {
        /// The length of the provided RGBA data.
        byte_count: usize,
    },
    /// Produced when the number of pixels (`rgba.len() / 4`) isn't equal to `width * height`.
    /// At least one of your arguments is incorrect.
    #[error(
        "The number of RGBA pixels ({pixel_count}) does not match the \
        provided dimensions ({width}x{height})."
    )]
    DimensionsVsPixelCount {
        /// The provided width.
        width: u32,
        /// The provided height.
        height: u32,
        /// The product of `width` and `height`.
        width_x_height: usize,
        /// The amount of pixels of the provided RGBA data.
        pixel_count: usize,
    },
}
