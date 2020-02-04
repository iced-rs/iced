use crate::{Bus, Css, Element, Length, Widget};

use dodrio::bumpalo;

/// A frame that displays an image while keeping aspect ratio.
///
/// # Example
///
/// ```
/// # use iced_web::Image;
///
/// let image = Image::new("resources/ferris.png");
/// ```
#[derive(Debug)]
pub struct Image {
    /// The image path
    pub path: String,

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
            width: Length::Shrink,
            height: Length::Shrink,
        }
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

impl<Message> Widget<Message> for Image {
    fn node<'b>(
        &self,
        bump: &'b bumpalo::Bump,
        _bus: &Bus<Message>,
        _style_sheet: &mut Css<'b>,
    ) -> dodrio::Node<'b> {
        use dodrio::builder::*;

        let src = bumpalo::format!(in bump, "{}", self.path);

        let mut image = img(bump).attr("src", src.into_bump_str());

        match self.width {
            Length::Shrink => {}
            Length::Fill | Length::FillPortion(_) => {
                image = image.attr("width", "100%");
            }
            Length::Units(px) => {
                image = image.attr(
                    "width",
                    bumpalo::format!(in bump, "{}px", px).into_bump_str(),
                );
            }
        }

        // TODO: Complete styling

        image.finish()
    }
}

impl<'a, Message> From<Image> for Element<'a, Message> {
    fn from(image: Image) -> Element<'a, Message> {
        Element::new(image)
    }
}
