//! Load and draw raster graphics.
pub use bytes::Bytes;

use crate::{Rectangle, Size};

use rustc_hash::FxHasher;
use std::hash::{Hash, Hasher};
use std::path::PathBuf;

/// A handle of some image data.
#[derive(Clone, PartialEq, Eq)]
pub enum Handle {
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

impl Handle {
    /// Creates an image [`Handle`] pointing to the image of the given path.
    ///
    /// Makes an educated guess about the image format by examining the data in the file.
    pub fn from_path<T: Into<PathBuf>>(path: T) -> Handle {
        Self::Path(path.into())
    }

    /// Creates an image [`Handle`] containing the image pixels directly. This
    /// function expects the input data to be provided as a `Vec<u8>` of RGBA
    /// pixels.
    ///
    /// This is useful if you have already decoded your image.
    pub fn from_pixels(
        width: u32,
        height: u32,
        pixels: impl Into<Bytes>,
    ) -> Handle {
        Self::Rgba {
            width,
            height,
            pixels: pixels.into(),
        }
    }

    /// Creates an image [`Handle`] containing the image data directly.
    ///
    /// Makes an educated guess about the image format by examining the given data.
    ///
    /// This is useful if you already have your image loaded in-memory, maybe
    /// because you downloaded or generated it procedurally.
    pub fn from_memory(bytes: impl Into<Bytes>) -> Handle {
        Self::Bytes(bytes.into())
    }

    /// Returns the unique identifier of the [`Handle`].
    pub fn id(&self) -> u64 {
        let mut hasher = FxHasher::default();
        self.hash(&mut hasher);

        hasher.finish()
    }
}

impl Hash for Handle {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            Self::Path(path) => path.hash(state),
            Self::Bytes(bytes) => bytes.as_ptr().hash(state),
            Self::Rgba { pixels, .. } => pixels.as_ptr().hash(state),
        }
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

impl std::fmt::Debug for Handle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Path(path) => write!(f, "Path({path:?})"),
            Self::Bytes(_) => write!(f, "Bytes(...)"),
            Self::Rgba { width, height, .. } => {
                write!(f, "Pixels({width} * {height})")
            }
        }
    }
}

/// Image filtering strategy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FilterMethod {
    /// Bilinear interpolation.
    #[default]
    Linear,
    /// Nearest neighbor.
    Nearest,
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
    fn measure_image(&self, handle: &Self::Handle) -> Size<u32>;

    /// Draws an image with the given [`Handle`] and inside the provided
    /// `bounds`.
    fn draw_image(
        &mut self,
        handle: Self::Handle,
        filter_method: FilterMethod,
        bounds: Rectangle,
    );
}
