//! Attach an icon to the window of your application.
use std::fmt;
use std::io;

#[cfg(feature = "image_rs")]
use std::path::Path;

/// Adds a method to create an icon from an image file or file data.
pub trait IconExtend {
    /// Creates an icon from an image file.
    ///
    /// This will return an error in case the file is missing at run-time. You may prefer [`Self::from_file_data`] instead.
    fn from_file<P: AsRef<Path>>(icon_path: P) -> Result<Self, Error>
    where
        Self: Sized;

    /// Creates an icon from the content of an image file.
    ///
    /// This content can be included in your application at compile-time, e.g. using the `include_bytes!` macro. \
    /// You can pass an explicit file format. Otherwise, the file format will be guessed at runtime.
    fn from_file_data(
        data: &[u8],
        explicit_format: Option<image_rs::ImageFormat>,
    ) -> Result<Self, Error>
    where
        Self: Sized;
}

impl IconExtend for iced_native::window::Icon {
    #[cfg(feature = "image_rs")]
    fn from_file<P: AsRef<Path>>(icon_path: P) -> Result<Self, Error> {
        let icon = image_rs::io::Reader::open(icon_path)?.decode()?.to_rgba8();

        Self::from_rgba(icon.to_vec(), icon.width(), icon.height())
            .map_err(Error::IconError)
    }
    #[cfg(feature = "image_rs")]
    fn from_file_data(
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
            .map_err(|e| Error::IconError(e))
    }
}

/// An error produced when using `Icon::from_rgba` with invalid arguments.
#[derive(Debug)]
pub enum Error {
    /// The icon creation errors
    IconError(iced_native::window::BadIcon),

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

impl From<iced_native::window::BadIcon> for Error {
    fn from(error: iced_native::window::BadIcon) -> Self {
        Error::IconError(error)
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
            Error::IconError(e) => write!(f, "Unable to create icon: {}", e),
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
