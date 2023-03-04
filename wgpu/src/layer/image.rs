use crate::core::image;
use crate::core::svg;
use crate::core::{Color, Rectangle};

/// A raster or vector image.
#[derive(Debug, Clone)]
pub enum Image {
    /// A raster image.
    Raster {
        /// The handle of a raster image.
        handle: image::Handle,

        /// The bounds of the image.
        bounds: Rectangle,
    },
    /// A vector image.
    Vector {
        /// The handle of a vector image.
        handle: svg::Handle,

        /// The [`Color`] filter
        color: Option<Color>,

        /// The bounds of the image.
        bounds: Rectangle,
    },
}
