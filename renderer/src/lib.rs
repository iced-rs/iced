pub mod window;

mod backend;
mod settings;

pub use backend::Backend;
pub use settings::Settings;

pub use iced_graphics::{
    Antialiasing, Color, Error, Font, Point, Size, Viewport,
};

/// The default graphics renderer for [`iced`].
///
/// [`iced`]: https://github.com/iced-rs/iced
pub type Renderer<Theme = iced_native::Theme> =
    iced_graphics::Renderer<Backend, Theme>;
