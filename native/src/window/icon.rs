use std::{fmt::Display, mem};

/// An icon used for the window titlebar, taskbar, etc.
#[repr(C)]
#[derive(Debug)]
struct Pixel {
    pub(crate) r: u8,
    pub(crate) g: u8,
    pub(crate) b: u8,
    pub(crate) a: u8,
}

const PIXEL_SIZE: usize = mem::size_of::<Pixel>();

/// icon
#[derive(Debug, Clone)]
pub struct Icon {
    rgba: Vec<u8>,
    width: u32,
    height: u32,
}

impl Icon {
    /// return the icon data
    pub fn into_raw(self) -> (Vec<u8>, u32, u32) {
        (self.rgba, self.width, self.height)
    }

    /// build Icon from rgba pixels
    pub fn from_rgba(
        rgba: Vec<u8>,
        width: u32,
        height: u32,
    ) -> Result<Self, BadIcon> {
        if rgba.len() % PIXEL_SIZE != 0 {
            return Err(BadIcon::ByteCountNotDivisibleBy4 {
                byte_count: rgba.len(),
            });
        }
        let pixel_count = rgba.len() / PIXEL_SIZE;
        if pixel_count != (width * height) as usize {
            return Err(BadIcon::DimensionsVsPixelCount {
                width,
                height,
                width_x_height: (width * height) as usize,
                pixel_count,
            });
        }

        Ok(Icon {
            rgba,
            width,
            height,
        })
    }
}

#[derive(Debug)]
/// An error produced when using [`Icon::from_rgba`] with invalid arguments.
pub enum BadIcon {
    /// Produced when the length of the `rgba` argument isn't divisible by 4, thus `rgba` can't be
    /// safely interpreted as 32bpp RGBA pixels.
    ByteCountNotDivisibleBy4 {
        /// The length of the provided RGBA data.
        byte_count: usize,
    },
    /// Produced when the number of pixels (`rgba.len() / 4`) isn't equal to `width * height`.
    /// At least one of your arguments is incorrect.
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

impl Display for BadIcon {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BadIcon::ByteCountNotDivisibleBy4 { byte_count } => {
                write!(
                    f,
                    "The provided RGBA data (with length {byte_count:?}) isn't divisble by \
                4. Therefore, it cannot be safely interpreted as 32bpp RGBA \
                pixels."
                )
            }
            BadIcon::DimensionsVsPixelCount {
                width,
                height,
                pixel_count,
                ..
            } => {
                write!(
                    f,
                    "The number of RGBA pixels ({pixel_count:?}) does not match the provided \
                dimensions ({width:?}x{height:?})."
                )
            }
        }
    }
}
