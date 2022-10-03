//! Load and draw raster graphics.
use crate::{Hasher, Rectangle};

use std::borrow::Cow;
use std::hash::{Hash, Hasher as _};
use std::path::PathBuf;
use std::sync::Arc;

/// A handle of some image data.
#[derive(Debug, Clone)]
pub struct Handle {
    id: u64,
    data: Arc<Data>,
}

impl Handle {
    /// Creates an image [`Handle`] pointing to the image of the given path.
    ///
    /// Makes an educated guess about the image format by examining the data in the file.
    pub fn from_path<T: Into<PathBuf>>(path: T) -> Handle {
        Self::from_data(Data::Path(path.into()))
    }

    /// Creates an image [`Handle`] containing the image pixels directly. This
    /// function expects the input data to be provided as a `Vec<u8>` of BGRA
    /// pixels.
    ///
    /// This is useful if you have already decoded your image.
    pub fn from_pixels(width: u32, height: u32, pixels: Vec<u8>) -> Handle {
        Self::from_data(Data::Pixels {
            width,
            height,
            pixels: Cow::Owned(pixels),
        })
    }

    /// Like [`Handle::from_pixels`], but from static pixel data.
    ///
    /// Useful for images included in binary, for instance with [`include_bytes!`].
    pub fn from_static_pixels(
        width: u32,
        height: u32,
        pixels: &'static [u8],
    ) -> Handle {
        Self::from_data(Data::Pixels {
            width,
            height,
            pixels: Cow::Borrowed(pixels),
        })
    }

    /// Creates an image [`Handle`] containing the image data directly.
    ///
    /// Makes an educated guess about the image format by examining the given data.
    ///
    /// This is useful if you already have your image loaded in-memory, maybe
    /// because you downloaded or generated it procedurally.
    pub fn from_memory(bytes: Vec<u8>) -> Handle {
        Self::from_data(Data::Bytes(Cow::Owned(bytes)))
    }

    /// Like [`Handle::from_memory`], but from static image data.
    ///
    /// Useful for images included in binary, for instance with [`include_bytes!`].
    pub fn from_static_memory(bytes: &'static [u8]) -> Handle {
        Self::from_data(Data::Bytes(Cow::Borrowed(bytes)))
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

/// The data of a raster image.
#[derive(Clone, Hash)]
pub enum Data {
    /// File data
    Path(PathBuf),

    /// In-memory data
    Bytes(Cow<'static, [u8]>),

    /// Decoded image pixels in BGRA format.
    Pixels {
        /// The width of the image.
        width: u32,
        /// The height of the image.
        height: u32,
        /// The pixels.
        pixels: Cow<'static, [u8]>,
    },
}

impl std::fmt::Debug for Data {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Data::Path(path) => write!(f, "Path({:?})", path),
            Data::Bytes(_) => write!(f, "Bytes(...)"),
            Data::Pixels { width, height, .. } => {
                write!(f, "Pixels({} * {})", width, height)
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
    fn dimensions(&self, handle: &Self::Handle) -> (u32, u32);

    /// Draws an image with the given [`Handle`] and inside the provided
    /// `bounds`.
    fn draw(&mut self, handle: Self::Handle, bounds: Rectangle);
}
