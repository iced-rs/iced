//! Build and draw geometry.
pub mod fill;
pub mod path;
pub mod stroke;

mod style;
mod text;

pub use fill::Fill;
pub use path::Path;
pub use stroke::{LineCap, LineDash, LineJoin, Stroke};
pub use style::Style;
pub use text::Text;

pub use crate::gradient::{self, Gradient};

/// A renderer capable of drawing some [`Self::Geometry`].
pub trait Renderer: crate::core::Renderer {
    /// The kind of geometry this renderer can draw.
    type Geometry;

    /// Draws the given layers of [`Self::Geometry`].
    fn draw(&mut self, layers: Vec<Self::Geometry>);
}
