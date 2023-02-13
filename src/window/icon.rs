//! Attach an icon to the window of your application.
use std::fmt;
use std::io;

#[cfg(feature = "image_rs")]
use std::path::Path;

/// The icon of a window.
#[derive(Debug, Clone)]
pub struct Icon(iced_winit::winit::window::Icon);

impl Icon {
    /// Creates an icon from 32bpp RGBA data.
    pub fn from_rgba(
        rgba: Vec<u8>,
        width: u32,
        height: u32,
    ) -> Result<Self, Error> {
        let raw =
            iced_winit::winit::window::Icon::from_rgba(rgba, width, height)?;

        Ok(Icon(raw))
    }

    /// Creates an icon from an image file.
    ///
    /// This will return an error in case the file is missing at run-time. You may prefer [`Self::from_file_data`] instead.
    #[cfg(feature = "image_rs")]
    pub fn from_file<P: AsRef<Path>>(icon_path: P) -> Result<Self, Error> {
        let icon = image_rs::io::Reader::open(icon_path)?.decode()?.to_rgba8();

        Self::from_rgba(icon.to_vec(), icon.width(), icon.height())
    }

    /// Creates an icon from the content of an image file.
    ///
    /// This content can be included in your application at compile-time, e.g. using the `include_bytes!` macro. \
    /// You can pass an explicit file format. Otherwise, the file format will be guessed at runtime.
    #[cfg(feature = "image_rs")]
    pub fn from_file_data(
        data: &[u8],
        explicit_format: Option<image_rs::ImageFormat>,
    ) -> Result<Self, Error> {
        let mut icon = image_rs::io::Reader::new(std::io::Cursor::new(data));
        let icon_with_format = match explicit_format {
            Some(format) => {
                icon.set_format(format);
                icon
            }
            None => icon.with_guessed_format()?,
        };

        let pixels = icon_with_format.decode()?.to_rgba8();

        Self::from_rgba(pixels.to_vec(), pixels.width(), pixels.height())
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

    /// The `image` crate reported an error
    #[cfg(feature = "image_rs")]
    ImageError(image_rs::error::ImageError),
}

impl From<std::io::Error> for Error {
    fn from(os_error: std::io::Error) -> Self {
        Error::OsError(os_error)
    }
}

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

impl From<Icon> for iced_winit::winit::window::Icon {
    fn from(icon: Icon) -> Self {
        icon.0
    }
}

#[cfg(feature = "image_rs")]
impl From<image_rs::error::ImageError> for Error {
    fn from(image_error: image_rs::error::ImageError) -> Self {
        Self::ImageError(image_error)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::InvalidData { byte_count } => {
                write!(
                    f,
                    "The provided RGBA data (with length {byte_count:?}) isn't divisble by \
                4. Therefore, it cannot be safely interpreted as 32bpp RGBA \
                pixels."
                )
            }
            Error::DimensionsMismatch {
                width,
                height,
                pixel_count,
            } => {
                write!(
                    f,
                    "The number of RGBA pixels ({pixel_count:?}) does not match the provided \
                dimensions ({width:?}x{height:?})."
                )
            }
            Error::OsError(e) => write!(
                f,
                "The underlying OS failed to create the window \
                icon: {e:?}"
            ),
            #[cfg(feature = "image_rs")]
            Error::ImageError(e) => {
                write!(f, "Unable to create icon from a file: {e:?}")
            }
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(self)
    }
}
