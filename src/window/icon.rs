//! Attach an icon to the window of your application.
pub use crate::runtime::window::icon::*;

use crate::runtime::window::icon;

use std::io;

#[cfg(feature = "image")]
use std::path::Path;

/// Creates an icon from an image file.
///
/// This will return an error in case the file is missing at run-time. You may prefer [`Self::from_file_data`] instead.
#[cfg(feature = "image")]
#[cfg_attr(docsrs, doc(cfg(feature = "image")))]
pub fn from_file<P: AsRef<Path>>(icon_path: P) -> Result<Icon, Error> {
    let icon = image_rs::io::Reader::open(icon_path)?.decode()?.to_rgba8();

    Ok(icon::from_rgba(icon.to_vec(), icon.width(), icon.height())?)
}

/// Creates an icon from the content of an image file.
///
/// This content can be included in your application at compile-time, e.g. using the `include_bytes!` macro. \
/// You can pass an explicit file format. Otherwise, the file format will be guessed at runtime.
#[cfg(feature = "image")]
#[cfg_attr(docsrs, doc(cfg(feature = "image")))]
pub fn from_file_data(
    data: &[u8],
    explicit_format: Option<image_rs::ImageFormat>,
) -> Result<Icon, Error> {
    let mut icon = image_rs::io::Reader::new(std::io::Cursor::new(data));
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

/// An error produced when using `Icon::from_rgba` with invalid arguments.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// The icon creation errors
    #[error("Unable to create icon: {0}")]
    IconError(#[from] icon::Error),

    /// The underlying OS failed to create the icon.
    #[error("The underlying OS failted to create the window icon: {0}")]
    OsError(#[from] io::Error),

    /// The `image` crate reported an error
    #[cfg(feature = "image")]
    #[cfg_attr(docsrs, doc(cfg(feature = "image")))]
    #[error("Unable to create icon from a file: {0}")]
    ImageError(#[from] image_rs::error::ImageError),
}
