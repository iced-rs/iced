pub mod window;

mod backend;
mod settings;
mod text;

#[cfg(feature = "geometry")]
pub mod geometry;

pub use iced_graphics::primitive;

pub use backend::Backend;
pub use primitive::Primitive;
pub use settings::Settings;

pub use iced_graphics::{
    Color, Error, Font, Point, Rectangle, Size, Vector, Viewport,
};

/// A [`tiny-skia`] graphics renderer for [`iced`].
///
/// [`tiny-skia`]: https://github.com/RazrFalcon/tiny-skia
/// [`iced`]: https://github.com/iced-rs/iced
pub type Renderer<Theme = iced_native::Theme> =
    iced_graphics::Renderer<Backend, Theme>;
