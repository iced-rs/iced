//! Build and draw geometry.
pub mod fill;
pub mod frame;
pub mod path;
pub mod stroke;

mod cache;
mod style;
mod text;

pub use cache::Cache;
pub use fill::Fill;
pub use frame::Frame;
pub use path::Path;
pub use stroke::{LineCap, LineDash, LineJoin, Stroke};
pub use style::Style;
pub use text::Text;

pub use crate::gradient::{self, Gradient};

use crate::core::{self, Size};
use crate::Cached;

/// A renderer capable of drawing some [`Self::Geometry`].
pub trait Renderer: core::Renderer {
    /// The kind of geometry this renderer can draw.
    type Geometry: Cached;

    /// The kind of [`Frame`] this renderer supports.
    type Frame: frame::Backend<Geometry = Self::Geometry>;

    /// Creates a new [`Self::Frame`].
    fn new_frame(&self, size: Size) -> Self::Frame;

    /// Draws the given [`Self::Geometry`].
    fn draw_geometry(&mut self, geometry: Self::Geometry);
}

/// The graphics backend of a geometry renderer.
pub trait Backend {
    /// The kind of [`Frame`] this backend supports.
    type Frame: frame::Backend;

    /// Creates a new [`Self::Frame`].
    fn new_frame(&self, size: Size) -> Self::Frame;
}
