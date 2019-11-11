//! Display images in your user interface.

use crate::{Length, Rectangle};

/// A frame that displays an image while keeping aspect ratio.
///
/// # Example
///
/// ```
/// use iced_core::Image;
///
/// let image = Image::new("resources/ferris.png");
/// ```
#[derive(Debug)]
pub struct Image {
    /// The image path
    pub path: String,

    /// The part of the image to show
    pub clip: Option<Rectangle<u16>>,

    /// The width of the image
    pub width: Length,

    /// The height of the image
    pub height: Length,
}

impl Image {
    /// Creates a new [`Image`] with the given path.
    ///
    /// [`Image`]: struct.Image.html
    pub fn new<T: Into<String>>(path: T) -> Self {
        Image {
            path: path.into(),
            clip: None,
            width: Length::Shrink,
            height: Length::Shrink,
        }
    }

    /// Sets the portion of the [`Image`] to draw.
    ///
    /// [`Image`]: struct.Image.html
    pub fn clip(mut self, clip: Rectangle<u16>) -> Self {
        self.clip = Some(clip);
        self
    }

    /// Sets the width of the [`Image`] boundaries.
    ///
    /// [`Image`]: struct.Image.html
    pub fn width(mut self, width: Length) -> Self {
        self.width = width;
        self
    }

    /// Sets the height of the [`Image`] boundaries.
    ///
    /// [`Image`]: struct.Image.html
    pub fn height(mut self, height: Length) -> Self {
        self.height = height;
        self
    }
}
