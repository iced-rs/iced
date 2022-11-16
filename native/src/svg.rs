//! Load and draw vector graphics.
use crate::{Hasher, Rectangle, Size};

use std::borrow::Cow;
use std::hash::{Hash, Hasher as _};
use std::path::PathBuf;
use std::sync::Arc;

pub use iced_style::svg::{Appearance, StyleSheet};

/// A handle of Svg data.
#[derive(Debug, Clone)]
pub struct Handle {
    id: u64,
    data: Arc<Data>,
    appearance: Appearance,
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
    pub fn from_memory(bytes: impl Into<Cow<'static, [u8]>>) -> Handle {
        Self::from_data(Data::Bytes(bytes.into()))
    }

    fn from_data(data: Data) -> Handle {
        let mut hasher = Hasher::default();
        data.hash(&mut hasher);

        Handle {
            id: hasher.finish(),
            data: Arc::new(data),
            appearance: Appearance::default(),
        }
    }

    /// Returns the unique identifier of the [`Handle`].
    pub fn id(&self) -> u64 {
        self.id
    }

    /// Returns a reference to the SVG [`Data`].
    pub fn data(&self) -> &Data {
        &self.data
    }

    /// Returns the styling [`Appearance`] for the SVG.
    pub fn appearance(&self) -> Appearance {
        self.appearance
    }

    /// Set the [`Appearance`] for the SVG.
    pub fn set_appearance(&mut self, appearance: Appearance) {
        self.appearance = appearance;
    }
}

impl Hash for Handle {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

/// The data of a vectorial image.
#[derive(Clone, Hash)]
pub enum Data {
    /// File data
    Path(PathBuf),

    /// In-memory data
    ///
    /// Can contain an SVG string or a gzip compressed data.
    Bytes(Cow<'static, [u8]>),
}

impl std::fmt::Debug for Data {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Data::Path(path) => write!(f, "Path({:?})", path),
            Data::Bytes(_) => write!(f, "Bytes(...)"),
        }
    }
}

/// A [`Renderer`] that can render vector graphics.
///
/// [renderer]: crate::renderer
pub trait Renderer: crate::Renderer {
    /// Returns the default dimensions of an SVG for the given [`Handle`].
    fn dimensions(&self, handle: &Handle) -> Size<u32>;

    /// Draws an SVG with the given [`Handle`] and inside the provided `bounds`.
    fn draw(&mut self, handle: Handle, bounds: Rectangle);
}
