//! Produce, store, and reuse geometry.
mod cache;

pub use cache::Cache;

use crate::triangle;

use iced_native::Size;
use std::sync::Arc;

/// A layer that can be presented at a [`Canvas`].
///
/// [`Canvas`]: ../struct.Canvas.html
pub trait Layer: std::fmt::Debug {
    /// Draws the [`Layer`] in the given bounds and produces [`Mesh2D`] as a
    /// result.
    ///
    /// The [`Layer`] may choose to store the produced [`Mesh2D`] locally and
    /// only recompute it when the bounds change, its contents change, or is
    /// otherwise explicitly cleared by other means.
    ///
    /// [`Layer`]: trait.Layer.html
    /// [`Mesh2D`]: ../../../triangle/struct.Mesh2D.html
    fn draw(&self, bounds: Size) -> Arc<triangle::Mesh2D>;
}
