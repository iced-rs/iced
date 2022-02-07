//! Load and draw raster graphics.
use crate::Rectangle;

use std::hash::Hash;

pub use iced_core::image::Data;
pub use iced_core::image::Handle;

/// A [`Renderer`] that can render raster graphics.
///
/// [renderer]: crate::renderer
pub trait Renderer: crate::Renderer {
    /// The image Handle to be displayed. Iced exposes its own default implementation of a [`Handle`]
    ///
    /// [`Handle`]: Self::Handle
    type Handle: Clone + Hash;

    /// Returns the dimensions of an image for the given [`Handle`].
    fn dimensions(&self, handle: &Self::Handle) -> (u32, u32);

    /// Draws an image with the given [`Handle`] and inside the provided
    /// `bounds`.
    fn draw(&mut self, handle: Self::Handle, bounds: Rectangle);
}
