//! Load and draw raster graphics.
pub use bytes::Bytes;

use crate::{Radians, Rectangle, Size};

use rustc_hash::FxHasher;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};

/// A raster image that can be drawn.
#[derive(Debug, Clone, PartialEq)]
pub struct Image<H = Handle> {
    /// The handle of the image.
    pub handle: H,

    /// The filter method of the image.
    pub filter_method: FilterMethod,

    /// The rotation to be applied to the image; on its center.
    pub rotation: Radians,

    /// The opacity of the image.
    ///
    /// 0 means transparent. 1 means opaque.
    pub opacity: f32,

    /// If set to `true`, the image will be snapped to the pixel grid.
    ///
    /// This can avoid graphical glitches, specially when using
    /// [`FilterMethod::Nearest`].
    pub snap: bool,
}

impl Image<Handle> {
    /// Creates a new [`Image`] with the given handle.
    pub fn new(handle: impl Into<Handle>) -> Self {
        Self {
            handle: handle.into(),
            filter_method: FilterMethod::default(),
            rotation: Radians(0.0),
            opacity: 1.0,
            snap: false,
        }
    }

    /// Sets the filter method of the [`Image`].
    pub fn filter_method(mut self, filter_method: FilterMethod) -> Self {
        self.filter_method = filter_method;
        self
    }

    /// Sets the rotation of the [`Image`].
    pub fn rotation(mut self, rotation: impl Into<Radians>) -> Self {
        self.rotation = rotation.into();
        self
    }

    /// Sets the opacity of the [`Image`].
    pub fn opacity(mut self, opacity: impl Into<f32>) -> Self {
        self.opacity = opacity.into();
        self
    }

    /// Sets whether the [`Image`] should be snapped to the pixel grid.
    pub fn snap(mut self, snap: bool) -> Self {
        self.snap = snap;
        self
    }
}

impl From<&Handle> for Image {
    fn from(handle: &Handle) -> Self {
        Image::new(handle.clone())
    }
}

/// A handle of some image data.
#[derive(Clone, PartialEq, Eq)]
pub enum Handle {
    /// A file handle. The image data will be read
    /// from the file path.
    ///
    /// Use [`from_path`] to create this variant.
    ///
    /// [`from_path`]: Self::from_path
    Path(Id, PathBuf),

    /// A handle pointing to some encoded image bytes in-memory.
    ///
    /// Use [`from_bytes`] to create this variant.
    ///
    /// [`from_bytes`]: Self::from_bytes
    Bytes(Id, Bytes),

    /// A handle pointing to decoded image pixels in RGBA format.
    ///
    /// Use [`from_rgba`] to create this variant.
    ///
    /// [`from_rgba`]: Self::from_rgba
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

    /// Creates an image [`Handle`] containing the encoded image data directly.
    ///
    /// Makes an educated guess about the image format by examining the given data.
    ///
    /// This is useful if you already have your image loaded in-memory, maybe
    /// because you downloaded or generated it procedurally.
    pub fn from_bytes(bytes: impl Into<Bytes>) -> Handle {
        Self::Bytes(Id::unique(), bytes.into())
    }

    /// Creates an image [`Handle`] containing the decoded image pixels directly.
    ///
    /// This function expects the pixel data to be provided as a collection of [`Bytes`]
    /// of RGBA pixels. Therefore, the length of the pixel data should always be
    /// `width * height * 4`.
    ///
    /// This is useful if you have already decoded your image.
    pub fn from_rgba(
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

impl From<&Handle> for Handle {
    fn from(value: &Handle) -> Self {
        value.clone()
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
pub struct Id(_Id);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum _Id {
    Unique(u64),
    Hash(u64),
}

impl Id {
    fn unique() -> Self {
        use std::sync::atomic::{self, AtomicU64};

        static NEXT_ID: AtomicU64 = AtomicU64::new(0);

        Self(_Id::Unique(NEXT_ID.fetch_add(1, atomic::Ordering::Relaxed)))
    }

    fn path(path: impl AsRef<Path>) -> Self {
        let hash = {
            let mut hasher = FxHasher::default();
            path.as_ref().hash(&mut hasher);

            hasher.finish()
        };

        Self(_Id::Hash(hash))
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

    /// Draws an [`Image`] inside the provided `bounds`.
    fn draw_image(&mut self, image: Image<Self::Handle>, bounds: Rectangle);
}
