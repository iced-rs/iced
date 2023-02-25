pub mod window;

mod backend;
mod settings;

pub use backend::Backend;
pub use settings::Settings;

pub use iced_graphics::{Color, Error, Font, Point, Size, Vector, Viewport};

/// A [`tiny-skia`] graphics renderer for [`iced`].
///
/// [`tiny-skia`]: https://github.com/RazrFalcon/tiny-skia
/// [`iced`]: https://github.com/iced-rs/iced
pub type Renderer<Theme = iced_native::Theme> =
    iced_graphics::Renderer<Backend, Theme>;
