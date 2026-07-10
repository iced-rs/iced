//! Load and draw vector graphics.
use crate::{Bytes, Color, Radians, Rectangle, Size};

use resvg::usvg;
use rustc_hash::FxHasher;
use std::fmt::{self, Debug};
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::PathBuf;
use std::sync::Arc;

/// The unique identifier of some [`Handle`] data.
#[derive(Clone)]
pub enum Id {
    /// Hash value of [`Data`]
    Hash(u64),
    /// Address of allocated [`usvg::Tree`]
    Addr(Arc<usvg::Tree>),
}

impl Debug for Id {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            &Self::Hash(hash) => write!(f, "{}", hash),
            Self::Addr(addr) => write!(f, "{:?}", addr.as_ref() as *const usvg::Tree),
        }
    }
}

impl PartialEq for Id {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (&Self::Hash(x), &Self::Hash(y)) => x == y,
            (Self::Addr(x), Self::Addr(y)) => {
                (x.as_ref() as *const usvg::Tree) == (y.as_ref() as *const usvg::Tree)
            }
            _ => false,
        }
    }
}

impl Eq for Id {}

impl Hash for Id {
    fn hash<H: Hasher>(&self, state: &mut H) {
        core::mem::discriminant(self).hash(state);
        match self {
            &Id::Hash(x) => state.write_u64(x),
            Id::Addr(tree) => (tree.as_ref() as *const usvg::Tree).hash(state),
        }
    }
}

/// A raster image that can be drawn.
#[derive(Debug, Clone, PartialEq)]
pub struct Svg<H = Handle> {
    /// The handle of the [`Svg`].
    pub handle: H,

    /// The [`Color`] filter to be applied to the [`Svg`].
    ///
    /// If some [`Color`] is set, the whole [`Svg`] will be
    /// painted with it—ignoring any intrinsic colors.
    ///
    /// This can be useful for coloring icons programmatically
    /// (e.g. with a theme).
    pub color: Option<Color>,

    /// The rotation to be applied to the image; on its center.
    pub rotation: Radians,

    /// The opacity of the [`Svg`].
    ///
    /// 0 means transparent. 1 means opaque.
    pub opacity: f32,
}

impl Svg<Handle> {
    /// Creates a new [`Svg`] with the given handle.
    pub fn new(handle: impl Into<Handle>) -> Self {
        Self {
            handle: handle.into(),
            color: None,
            rotation: Radians(0.0),
            opacity: 1.0,
        }
    }

    /// Sets the [`Color`] filter of the [`Svg`].
    pub fn color(mut self, color: impl Into<Color>) -> Self {
        self.color = Some(color.into());
        self
    }

    /// Sets the rotation of the [`Svg`].
    pub fn rotation(mut self, rotation: impl Into<Radians>) -> Self {
        self.rotation = rotation.into();
        self
    }

    /// Sets the opacity of the [`Svg`].
    pub fn opacity(mut self, opacity: impl Into<f32>) -> Self {
        self.opacity = opacity.into();
        self
    }
}

impl From<&Handle> for Svg {
    fn from(handle: &Handle) -> Self {
        Svg::new(handle.clone())
    }
}

/// A handle of Svg data.
#[derive(Debug, Clone)]
pub enum Handle {
    /// Unloaded svg [`Data`]
    Unloaded {
        /// Hash value of [`Handle::Unloaded::data`]
        hash: u64,
        /// Data storage of a [`Handle`]
        data: Data,
    },

    /// Parsed [`usvg::Tree`]
    Loaded(Arc<usvg::Tree>),
}

impl Handle {
    /// Creates an SVG [`Handle`] pointing to the vector image of the given
    /// path.
    pub fn from_path(path: impl Into<PathBuf>) -> Handle {
        Self::from_data(Data::Path(path.into()))
    }

    /// Creates an SVG [`Handle`] from raw bytes containing either an SVG string
    /// or gzip compressed data.
    ///
    /// This is useful if you already have your SVG data in-memory, maybe
    /// because you downloaded or generated it procedurally.
    pub fn from_memory(bytes: impl Into<Bytes>) -> Handle {
        Self::from_data(Data::Bytes(bytes.into()))
    }

    fn from_data(data: Data) -> Handle {
        let mut hasher = FxHasher::default();
        data.hash(&mut hasher);

        Handle::Unloaded {
            hash: hasher.finish(),
            data,
        }
    }

    /// Creates an SVG [`Handle`] from a parsed `usvg::Tree`
    pub fn from_tree(tree: Arc<usvg::Tree>) -> Handle {
        Self::Loaded(tree)
    }

    /// Returns the unique identifier of the [`Handle`].
    pub fn id(&self) -> Id {
        match self {
            &Handle::Unloaded { hash, .. } => Id::Hash(hash),
            Handle::Loaded(tree) => Id::Addr(tree.clone()),
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

impl Hash for Handle {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            &Handle::Unloaded { hash, .. } => state.write_u64(hash),
            _ => {}
        }
    }
}

impl PartialEq for Handle {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (&Handle::Unloaded { hash: x, .. }, &Handle::Unloaded { hash: y, .. }) => x == y,
            _ => false,
        }
    }
}

/// The data of a vectorial image.
#[derive(Clone, Hash, PartialEq, Eq)]
pub enum Data {
    /// File data
    Path(PathBuf),

    /// In-memory data
    ///
    /// Can contain an SVG string or a gzip compressed data.
    Bytes(Bytes),
}

impl Data {
    /// Try to load and parse `Data` to `usvg::Tree`
    pub fn load(&self, options: &usvg::Options<'_>) -> Option<usvg::Tree> {
        match self {
            Self::Path(path) => fs::read_to_string(&path)
                .ok()
                .and_then(|text| usvg::Tree::from_str(&text, &options).ok()),
            Data::Bytes(bytes) => usvg::Tree::from_data(&bytes, options).ok(),
        }
    }
}

impl Debug for Data {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Path(path) => f.debug_tuple("Path").field(path).finish(),
            Self::Bytes(_) => f.write_str("Bytes(...)"),
        }
    }
}

/// A [`Renderer`] that can render vector graphics.
///
/// [renderer]: crate::renderer
pub trait Renderer: crate::Renderer {
    /// Returns the default dimensions of an SVG for the given [`Handle`].
    fn measure_svg(&self, handle: &Handle) -> Size<u32>;

    /// Draws an SVG with the given [`Handle`], an optional [`Color`] filter, and inside the provided `bounds`.
    fn draw_svg(&mut self, svg: Svg, bounds: Rectangle, clip_bounds: Rectangle);
}
