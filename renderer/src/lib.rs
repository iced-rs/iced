pub mod compositor;

#[cfg(feature = "geometry")]
pub mod geometry;

mod backend;
mod settings;

pub use iced_graphics as graphics;
pub use iced_graphics::core;

pub use backend::Backend;
pub use compositor::Compositor;
pub use settings::Settings;

/// The default graphics renderer for [`iced`].
///
/// [`iced`]: https://github.com/iced-rs/iced
pub type Renderer<Theme> = iced_graphics::Renderer<Backend, Theme>;
