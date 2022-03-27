//! Options for how to render images.

/// The algorithm to use when rendering a scaled Image
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum ImageFilter {
    /// Instructs the renderer to bilinearly interpolate nearby pixels, creating a smoother image
    Linear,

    /// Instructs the renderer to use the color of the nearest pixel without any interpolation. Preserves sharp edges between pixels, which can be desirable for things like pixel art.
    NearestNeighbor,
}

/// Options for how the renderer should render a scaled image
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub struct FilterOptions {
    /// The filter to use when rendering an up-scaled image
    pub mag_filter: ImageFilter,

    /// The filter to use when rendering a down-scaled image
    pub min_filter: ImageFilter,
}
