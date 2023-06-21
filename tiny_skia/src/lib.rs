pub mod window;

mod backend;
mod primitive;
mod settings;
mod text;

#[cfg(feature = "image")]
mod raster;

#[cfg(feature = "svg")]
mod vector;

#[cfg(feature = "geometry")]
pub mod geometry;

pub use iced_graphics as graphics;
pub use iced_graphics::core;

pub use backend::Backend;
pub use primitive::Primitive;
pub use settings::Settings;

/// A [`tiny-skia`] graphics renderer for [`iced`].
///
/// [`tiny-skia`]: https://github.com/RazrFalcon/tiny-skia
/// [`iced`]: https://github.com/iced-rs/iced
pub type Renderer<Theme> = iced_graphics::Renderer<Backend, Theme>;
