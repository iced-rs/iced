//! Display images in your user interface.
use crate::{Bus, Css, Element, Hasher, Length, Widget};

use dodrio::bumpalo;
use std::{
    hash::{Hash, Hasher as _},
    path::PathBuf,
    sync::Arc,
};

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
    pub handle: Handle,

    /// The alt text of the image
    pub alt: String,

    /// The width of the image
    pub width: Length,

    /// The height of the image
    pub height: Length,
}

impl Image {
    /// Creates a new [`Image`] with the given path.
    ///
    /// [`Image`]: struct.Image.html
    pub fn new<T: Into<Handle>>(handle: T) -> Self {
        Image {
            handle: handle.into(),
            alt: Default::default(),
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

    /// Sets the alt text of the [`Image`].
    ///
    /// [`Image`]: struct.Image.html
    pub fn alt(mut self, alt: impl Into<String>) -> Self {
        self.alt = alt.into();
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
        use dodrio::bumpalo::collections::String;

        let src = String::from_str_in(
            match self.handle.data.as_ref() {
                Data::Path(path) => path.to_str().unwrap_or(""),
            },
            bump,
        )
        .into_bump_str();

        let alt = String::from_str_in(&self.alt, bump).into_bump_str();

        let mut image = img(bump).attr("src", src).attr("alt", alt);

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

/// An [`Image`] handle.
///
/// [`Image`]: struct.Image.html
#[derive(Debug, Clone)]
pub struct Handle {
    id: u64,
    data: Arc<Data>,
}

impl Handle {
    /// Creates an image [`Handle`] pointing to the image of the given path.
    ///
    /// [`Handle`]: struct.Handle.html
    pub fn from_path<T: Into<PathBuf>>(path: T) -> Handle {
        Self::from_data(Data::Path(path.into()))
    }

    fn from_data(data: Data) -> Handle {
        let mut hasher = Hasher::default();
        data.hash(&mut hasher);

        Handle {
            id: hasher.finish(),
            data: Arc::new(data),
        }
    }

    /// Returns the unique identifier of the [`Handle`].
    ///
    /// [`Handle`]: struct.Handle.html
    pub fn id(&self) -> u64 {
        self.id
    }

    /// Returns a reference to the image [`Data`].
    ///
    /// [`Data`]: enum.Data.html
    pub fn data(&self) -> &Data {
        &self.data
    }
}

impl From<String> for Handle {
    fn from(path: String) -> Handle {
        Handle::from_path(path)
    }
}

impl From<&str> for Handle {
    fn from(path: &str) -> Handle {
        Handle::from_path(path)
    }
}

/// The data of an [`Image`].
///
/// [`Image`]: struct.Image.html
#[derive(Clone, Hash)]
pub enum Data {
    /// A remote image
    Path(PathBuf),
}

impl std::fmt::Debug for Data {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Data::Path(path) => write!(f, "Path({:?})", path),
        }
    }
}
