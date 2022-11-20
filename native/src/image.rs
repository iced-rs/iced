//! Load and draw raster graphics.
use crate::{Hasher, Rectangle, Size};

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
    /// function expects the input data to be provided as a `Vec<u8>` of RGBA
    /// pixels.
    ///
    /// This is useful if you have already decoded your image.
    pub fn from_pixels(
        width: u32,
        height: u32,
        pixels: impl AsRef<[u8]> + Clone + Send + Sync + 'static,
    ) -> Handle {
        Self::from_data(Data::Rgba {
            width,
            height,
            pixels: ImageBytes::new(pixels),
        })
    }

    /// Creates an image [`Handle`] containing the image data directly.
    ///
    /// Makes an educated guess about the image format by examining the given data.
    ///
    /// This is useful if you already have your image loaded in-memory, maybe
    /// because you downloaded or generated it procedurally.
    pub fn from_memory(
        bytes: impl AsRef<[u8]> + Clone + Send + Sync + 'static,
    ) -> Handle {
        Self::from_data(Data::Bytes(ImageBytes::new(bytes)))
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

/// Wrapper around raw image data.
///
/// Behaves like a `&[u8]`.
pub struct ImageBytes(Box<dyn ImageBytesTrait>);

trait ImageBytesTrait: AsRef<[u8]> + Send + Sync + 'static {
    fn clone_boxed(&self) -> Box<dyn ImageBytesTrait>;
}

impl<T: AsRef<[u8]> + Clone + Send + Sync + 'static> ImageBytesTrait for T {
    fn clone_boxed(&self) -> Box<dyn ImageBytesTrait> {
        Box::new(self.clone())
    }
}

impl ImageBytes {
    /// Creates a new `ImageBytes` around `data`.
    pub fn new(
        data: impl AsRef<[u8]> + Clone + Send + Sync + 'static,
    ) -> ImageBytes {
        Self(Box::new(data))
    }
}

impl Clone for ImageBytes {
    fn clone(&self) -> Self {
        ImageBytes(self.0.clone_boxed())
    }
}

impl std::fmt::Debug for ImageBytes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.as_ref().as_ref().fmt(f)
    }
}

impl std::hash::Hash for ImageBytes {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.as_ref().as_ref().hash(state);
    }
}

impl AsRef<[u8]> for ImageBytes {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref().as_ref()
    }
}

impl std::ops::Deref for ImageBytes {
    type Target = [u8];

    fn deref(&self) -> &[u8] {
        self.0.as_ref().as_ref()
    }
}

/// The data of a raster image.
#[derive(Clone, Hash)]
pub enum Data {
    /// File data
    Path(PathBuf),

    /// In-memory data
    Bytes(ImageBytes),

    /// Decoded image pixels in RGBA format.
    Rgba {
        /// The width of the image.
        width: u32,
        /// The height of the image.
        height: u32,
        /// The pixels.
        pixels: ImageBytes,
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
