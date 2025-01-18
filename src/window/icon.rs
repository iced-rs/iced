//! Attach an icon to the window of your application.
pub use crate::core::window::icon::*;

use crate::core::window::icon;

use std::io;

#[cfg(feature = "image")]
use std::path::Path;

/// Creates an icon from an image file.
///
/// This will return an error in case the file is missing at run-time. You may prefer [`from_file_data`] instead.
#[cfg(feature = "image")]
pub fn from_file<P: AsRef<Path>>(icon_path: P) -> Result<Icon, Error> {
    let icon = image::ImageReader::open(icon_path)?.decode()?.to_rgba8();

    Ok(icon::from_rgba(icon.to_vec(), icon.width(), icon.height())?)
}

/// Creates an icon from the content of an image file.
///
/// This content can be included in your application at compile-time, e.g. using the `include_bytes!` macro.
/// You can pass an explicit file format. Otherwise, the file format will be guessed at runtime.
#[cfg(feature = "image")]
pub fn from_file_data(
    data: &[u8],
    explicit_format: Option<image::ImageFormat>,
) -> Result<Icon, Error> {
    let mut icon = image::ImageReader::new(std::io::Cursor::new(data));

    let icon_with_format = match explicit_format {
        Some(format) => {
            icon.set_format(format);
            icon
        }
        None => icon.with_guessed_format()?,
    };

    let pixels = icon_with_format.decode()?.to_rgba8();

    Ok(icon::from_rgba(
        pixels.to_vec(),
        pixels.width(),
        pixels.height(),
    )?)
}

/// An error produced when creating an [`Icon`].
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// The [`Icon`] is not valid.
    #[error("The icon is invalid: {0}")]
    InvalidError(#[from] icon::Error),

    /// The underlying OS failed to create the icon.
    #[error("The underlying OS failed to create the window icon: {0}")]
    OsError(#[from] io::Error),

    /// The `image` crate reported an error.
    #[cfg(feature = "image")]
    #[error("Unable to create icon from a file: {0}")]
    ImageError(#[from] image::error::ImageError),
}
