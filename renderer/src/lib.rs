#![forbid(rust_2018_idioms)]
#![deny(unsafe_code, unused_results, rustdoc::broken_intra_doc_links)]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]
#[cfg(feature = "wgpu")]
pub use iced_wgpu as wgpu;

pub mod fallback;

mod settings;

pub use iced_graphics as graphics;
pub use iced_graphics::core;

#[cfg(feature = "geometry")]
pub use iced_graphics::geometry;

pub use settings::Settings;

/// The default graphics renderer for [`iced`].
///
/// [`iced`]: https://github.com/iced-rs/iced
#[cfg(not(feature = "wgpu"))]
pub type Renderer = iced_tiny_skia::Renderer;

/// The default graphics renderer for [`iced`].
///
/// [`iced`]: https://github.com/iced-rs/iced
#[cfg(feature = "wgpu")]
pub type Renderer =
    fallback::Renderer<iced_wgpu::Renderer, iced_tiny_skia::Renderer>;

/// The default graphics compositor for [`iced`].
///
/// [`iced`]: https://github.com/iced-rs/iced
#[cfg(not(feature = "wgpu"))]
pub type Compositor = iced_tiny_skia::window::Compositor;

/// The default graphics renderer for [`iced`].
///
/// [`iced`]: https://github.com/iced-rs/iced
#[cfg(feature = "wgpu")]
pub type Compositor = fallback::Compositor<
    iced_wgpu::window::Compositor,
    iced_tiny_skia::window::Compositor,
>;
