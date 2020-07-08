//! Attach an icon to the window of your application.
use std::fmt;
use std::io;

/// The icon of a window.
#[cfg(all(not(target_arch = "wasm32"), not(target_os = "ios")))]
#[derive(Debug, Clone)]
pub struct Icon(iced_winit::winit::window::Icon);

/// The icon of a window.
#[cfg(any(target_arch = "wasm32", target_os = "ios"))]
#[derive(Debug, Clone)]
pub struct Icon;

impl Icon {
    /// Creates an icon from 32bpp RGBA data.
    #[cfg(all(not(target_arch = "wasm32"), not(target_os = "ios")))]
    pub fn from_rgba(
        rgba: Vec<u8>,
        width: u32,
        height: u32,
    ) -> Result<Self, Error> {
        let raw =
            iced_winit::winit::window::Icon::from_rgba(rgba, width, height)?;

        Ok(Icon(raw))
    }

    /// Creates an icon from 32bpp RGBA data.
    #[cfg(any(target_arch = "wasm32", target_os = "ios"))]
    pub fn from_rgba(
        _rgba: Vec<u8>,
        _width: u32,
        _height: u32,
    ) -> Result<Self, Error> {
        Ok(Icon)
    }
}

/// An error produced when using `Icon::from_rgba` with invalid arguments.
#[derive(Debug)]
pub enum Error {
    /// The provided RGBA data isn't divisble by 4.
    ///
    /// Therefore, it cannot be safely interpreted as 32bpp RGBA pixels.
    InvalidData {
        /// The length of the provided RGBA data.
        byte_count: usize,
    },

    /// The number of RGBA pixels does not match the provided dimensions.
    DimensionsMismatch {
        /// The provided width.
        width: u32,
        /// The provided height.
        height: u32,
        /// The amount of pixels of the provided RGBA data.
        pixel_count: usize,
    },

    /// The underlying OS failed to create the icon.
    OsError(io::Error),
}

#[cfg(all(not(target_arch = "wasm32"), not(target_os = "ios")))]
impl From<iced_winit::winit::window::BadIcon> for Error {
    fn from(error: iced_winit::winit::window::BadIcon) -> Self {
        use iced_winit::winit::window::BadIcon;

        match error {
            BadIcon::ByteCountNotDivisibleBy4 { byte_count } => {
                Error::InvalidData { byte_count }
            }
            BadIcon::DimensionsVsPixelCount {
                width,
                height,
                pixel_count,
                ..
            } => Error::DimensionsMismatch {
                width,
                height,
                pixel_count,
            },
            BadIcon::OsError(os_error) => Error::OsError(os_error),
        }
    }
}

#[cfg(all(not(target_arch = "wasm32"), not(target_os = "ios")))]
impl From<Icon> for iced_winit::winit::window::Icon {
    fn from(icon: Icon) -> Self {
        icon.0
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::InvalidData { byte_count } => {
                write!(f,
                "The provided RGBA data (with length {:?}) isn't divisble by \
                4. Therefore, it cannot be safely interpreted as 32bpp RGBA \
                pixels.",
                byte_count,
            )
            }
            Error::DimensionsMismatch {
                width,
                height,
                pixel_count,
            } => {
                write!(f,
                "The number of RGBA pixels ({:?}) does not match the provided \
                dimensions ({:?}x{:?}).",
                width, height, pixel_count,
            )
            }
            Error::OsError(e) => write!(
                f,
                "The underlying OS failed to create the window \
                icon: {:?}",
                e
            ),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(self)
    }
}
