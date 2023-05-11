//! Draw 2D graphics for your users.
//!
//! A [`Canvas`] widget can be used to draw different kinds of 2D shapes in a
//! [`Frame`]. It can be used for animation, data visualization, game graphics,
//! and more!
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

pub use crate::core::gradient::{self, Gradient};

use crate::Primitive;

/// A bunch of shapes that can be drawn.
#[derive(Debug, Clone)]
pub struct Geometry(pub Primitive);

impl From<Geometry> for Primitive {
    fn from(geometry: Geometry) -> Self {
        geometry.0
    }
}

/// A renderer capable of drawing some [`Geometry`].
pub trait Renderer: crate::core::Renderer {
    /// Draws the given layers of [`Geometry`].
    fn draw(&mut self, layers: Vec<Geometry>);
}
