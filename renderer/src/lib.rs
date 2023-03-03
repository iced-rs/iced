pub mod window;

#[cfg(feature = "geometry")]
pub mod geometry;

mod backend;
mod settings;

pub use iced_graphics::primitive;

pub use backend::Backend;
pub use primitive::Primitive;
pub use settings::Settings;

pub use iced_graphics::{
    Antialiasing, Color, Error, Font, Point, Rectangle, Size, Vector, Viewport,
};

/// The default graphics renderer for [`iced`].
///
/// [`iced`]: https://github.com/iced-rs/iced
pub type Renderer<Theme = iced_native::Theme> =
    iced_graphics::Renderer<Backend, Theme>;
