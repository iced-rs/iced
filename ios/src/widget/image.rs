//! Display images in your user interface.
use crate::{Element, Hasher, Length, Widget, layout};

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
    fn hash_layout(&self, state: &mut Hasher) {
        struct Marker;
        std::any::TypeId::of::<Marker>().hash(state);
        self.width.hash(state);
        self.height.hash(state);
    }
    fn layout(
        &self,
        limits: &layout::Limits,
    ) -> layout::Node {
        todo!();
    }

    fn width(&self) -> Length {
        todo!();
    }

    fn height(&self) -> Length {
        todo!();
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
