//! Display images in your user interface.

use crate::{Align, Length, Rectangle};

/// A frame that displays an image while keeping aspect ratio.
///
/// # Example
///
/// ```
/// use iced_core::Image;
///
/// # let my_handle = String::from("some_handle");
/// let image = Image::new(my_handle);
/// ```
pub struct Image<I> {
    /// The image handle
    pub handle: I,

    /// The part of the image to show
    pub clip: Option<Rectangle<u16>>,

    /// The width of the image
    pub width: Length,

    /// The height of the image
    pub height: Length,

    pub align_self: Option<Align>,
}

impl<I> std::fmt::Debug for Image<I> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Image")
            .field("clip", &self.clip)
            .field("width", &self.width)
            .field("height", &self.height)
            .finish()
    }
}

impl<I> Image<I> {
    /// Creates a new [`Image`] with given image handle.
    ///
    /// [`Image`]: struct.Image.html
    pub fn new(handle: I) -> Self {
        Image {
            handle,
            clip: None,
            width: Length::Shrink,
            height: Length::Shrink,
            align_self: None,
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

    /// Sets the alignment of the [`Image`] itself.
    ///
    /// This is useful if you want to override the default alignment given by
    /// the parent container.
    ///
    /// [`Image`]: struct.Image.html
    pub fn align_self(mut self, align: Align) -> Self {
        self.align_self = Some(align);
        self
    }
}
