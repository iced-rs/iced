//! Load and draw raster graphics.
pub use bytes::Bytes;

use crate::{Rectangle, Size};

use rustc_hash::FxHasher;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};

/// A handle of some image data.
#[derive(Clone, PartialEq, Eq)]
pub enum Handle {
    /// File data
    Path(Id, PathBuf),

    /// In-memory data
    Bytes(Id, Bytes),

    /// Decoded image pixels in RGBA format.
    Rgba {
        /// The id of this handle.
        id: Id,
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
        let path = path.into();

        Self::Path(Id::path(&path), path)
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
            id: Id::unique(),
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
        Self::Bytes(Id::unique(), bytes.into())
    }

    /// Returns the unique identifier of the [`Handle`].
    pub fn id(&self) -> Id {
        match self {
            Handle::Path(id, _)
            | Handle::Bytes(id, _)
            | Handle::Rgba { id, .. } => *id,
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
            Self::Path(_, path) => write!(f, "Path({path:?})"),
            Self::Bytes(_, _) => write!(f, "Bytes(...)"),
            Self::Rgba { width, height, .. } => {
                write!(f, "Pixels({width} * {height})")
            }
        }
    }
}

/// The unique identifier of some [`Handle`] data.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Id {
    /// A unique identifier.
    Unique(u64),
    /// A hash identifier.
    Hash(u64),
}

impl Id {
    fn unique() -> Self {
        use std::sync::atomic::{self, AtomicU64};

        static NEXT_ID: AtomicU64 = AtomicU64::new(0);

        Self::Unique(NEXT_ID.fetch_add(1, atomic::Ordering::Relaxed))
    }

    fn path(path: impl AsRef<Path>) -> Self {
        let hash = {
            let mut hasher = FxHasher::default();
            path.as_ref().hash(&mut hasher);

            hasher.finish()
        };

        Self::Hash(hash)
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
    type Handle: Clone;

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
