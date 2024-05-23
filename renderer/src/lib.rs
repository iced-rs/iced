//! The official renderer for iced.
#![cfg_attr(docsrs, feature(doc_auto_cfg))]
#[cfg(feature = "wgpu")]
pub use iced_wgpu as wgpu;

pub mod fallback;

pub use iced_graphics as graphics;
pub use iced_graphics::core;

#[cfg(feature = "geometry")]
pub use iced_graphics::geometry;

/// The default graphics renderer for [`iced`].
///
/// [`iced`]: https://github.com/iced-rs/iced
pub type Renderer = renderer::Renderer;

/// The default graphics compositor for [`iced`].
///
/// [`iced`]: https://github.com/iced-rs/iced
pub type Compositor = renderer::Compositor;

#[cfg(all(feature = "wgpu", feature = "tiny-skia"))]
mod renderer {
    pub type Renderer = crate::fallback::Renderer<
        iced_wgpu::Renderer,
        iced_tiny_skia::Renderer,
    >;

    pub type Compositor = crate::fallback::Compositor<
        iced_wgpu::window::Compositor,
        iced_tiny_skia::window::Compositor,
    >;
}

#[cfg(all(feature = "wgpu", not(feature = "tiny-skia")))]
mod renderer {
    pub type Renderer = iced_wgpu::Renderer;
    pub type Compositor = iced_wgpu::window::Compositor;
}

#[cfg(all(not(feature = "wgpu"), feature = "tiny-skia"))]
mod renderer {
    pub type Renderer = iced_tiny_skia::Renderer;
    pub type Compositor = iced_tiny_skia::window::Compositor;
}

#[cfg(not(any(feature = "wgpu", feature = "tiny-skia")))]
mod renderer {
    #[cfg(not(debug_assertions))]
    compile_error!(
        "Cannot compile `iced_renderer` in release mode \
        without a renderer feature enabled. \
        Enable either the `wgpu` or `tiny-skia` feature, or both."
    );

    pub type Renderer = ();
    pub type Compositor = ();
}
