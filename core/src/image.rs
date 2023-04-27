//! Load and draw raster graphics.
use crate::{Hasher, Rectangle, Size};

use std::hash::{Hash, Hasher as _};
use std::path::PathBuf;
use std::sync::Arc;

/// A handle of some image data.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Handle {
    id: u64,
    data: Data,
}

impl Handle {
    /// Creates an image [`Handle`] pointing to the image of the given path.
    ///
    /// Makes an educated guess about the image format by examining the data in the file.
    pub fn from_path<T: Into<PathBuf>>(path: T) -> Handle {
        Self::from_data(Data::Path(path.into()))
    }

    /// Creates an image [`Handle`] containing the image pixels directly. This
    /// function expects the input data to be provided as a `Vec<u8>` of RGBA
    /// pixels.
    ///
    /// This is useful if you have already decoded your image.
    pub fn from_pixels(
        width: u32,
        height: u32,
        pixels: impl AsRef<[u8]> + Send + Sync + 'static,
    ) -> Handle {
        Self::from_data(Data::Rgba {
            width,
            height,
            pixels: Bytes::new(pixels),
        })
    }

    /// Creates an image [`Handle`] containing the image data directly.
    ///
    /// Makes an educated guess about the image format by examining the given data.
    ///
    /// This is useful if you already have your image loaded in-memory, maybe
    /// because you downloaded or generated it procedurally.
    pub fn from_memory(
        bytes: impl AsRef<[u8]> + Send + Sync + 'static,
    ) -> Handle {
        Self::from_data(Data::Bytes(Bytes::new(bytes)))
    }

    fn from_data(data: Data) -> Handle {
        let mut hasher = Hasher::default();
        data.hash(&mut hasher);

        Handle {
            id: hasher.finish(),
            data,
        }
    }

    /// Returns the unique identifier of the [`Handle`].
    pub fn id(&self) -> u64 {
        self.id
    }

    /// Returns a reference to the image [`Data`].
    pub fn data(&self) -> &Data {
        &self.data
    }
}

impl<T> From<T> for Handle
where
    T: Into<PathBuf>,
{
    fn from(path: T) -> Handle {
        Handle::from_path(path.into())
    }
}

impl Hash for Handle {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

/// A wrapper around raw image data.
///
/// It behaves like a `&[u8]`.
#[derive(Clone)]
pub struct Bytes(Arc<dyn AsRef<[u8]> + Send + Sync + 'static>);

impl Bytes {
    /// Creates new [`Bytes`] around `data`.
    pub fn new(data: impl AsRef<[u8]> + Send + Sync + 'static) -> Self {
        Self(Arc::new(data))
    }
}

impl std::fmt::Debug for Bytes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.as_ref().as_ref().fmt(f)
    }
}

impl std::hash::Hash for Bytes {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.as_ref().as_ref().hash(state);
    }
}

impl PartialEq for Bytes {
    fn eq(&self, other: &Self) -> bool {
        self.as_ref() == other.as_ref()
    }
}

impl Eq for Bytes {}

impl AsRef<[u8]> for Bytes {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref().as_ref()
    }
}

impl std::ops::Deref for Bytes {
    type Target = [u8];

    fn deref(&self) -> &[u8] {
        self.0.as_ref().as_ref()
    }
}

/// The data of a raster image.
#[derive(Clone, PartialEq, Eq, Hash)]
pub enum Data {
    /// File data
    Path(PathBuf),

    /// In-memory data
    Bytes(Bytes),

    /// Decoded image pixels in RGBA format.
    Rgba {
        /// The width of the image.
        width: u32,
        /// The height of the image.
        height: u32,
        /// The pixels.
        pixels: Bytes,
    },
}

impl std::fmt::Debug for Data {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Data::Path(path) => write!(f, "Path({path:?})"),
            Data::Bytes(_) => write!(f, "Bytes(...)"),
            Data::Rgba { width, height, .. } => {
                write!(f, "Pixels({width} * {height})")
            }
        }
    }
}

/// A [`Renderer`] that can render raster graphics.
///
/// [renderer]: crate::renderer
pub trait Renderer: crate::Renderer {
    /// The image Handle to be displayed. Iced exposes its own default implementation of a [`Handle`]
    ///
    /// [`Handle`]: Self::Handle
    type Handle: Clone + Hash;

    /// Returns the dimensions of an image for the given [`Handle`].
    fn dimensions(&self, handle: &Self::Handle) -> Size<u32>;

    /// Draws an image with the given [`Handle`] and inside the provided
    /// `bounds`.
    fn draw(&mut self, handle: Self::Handle, bounds: Rectangle);
}
