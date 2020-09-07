//! A [`wgpu`] renderer for [`iced_native`].
//!
//! ![`iced_wgpu` crate graph](https://github.com/hecrj/iced/blob/cae26cb7bc627f4a5b3bcf1cd023a0c552e8c65e/docs/graphs/wgpu.png?raw=true)
//!
//! For now, it is the default renderer of [Iced] in native platforms.
//!
//! [`wgpu`] supports most modern graphics backends: Vulkan, Metal, DX11, and
//! DX12 (OpenGL and WebGL are still WIP). Additionally, it will support the
//! incoming [WebGPU API].
//!
//! Currently, `iced_wgpu` supports the following primitives:
//! - Text, which is rendered using [`wgpu_glyph`]. No shaping at all.
//! - Quads or rectangles, with rounded borders and a solid background color.
//! - Clip areas, useful to implement scrollables or hide overflowing content.
//! - Images and SVG, loaded from memory or the file system.
//! - Meshes of triangles, useful to draw geometry freely.
//!
//! [Iced]: https://github.com/hecrj/iced
//! [`iced_native`]: https://github.com/hecrj/iced/tree/master/native
//! [`wgpu`]: https://github.com/gfx-rs/wgpu-rs
//! [WebGPU API]: https://gpuweb.github.io/gpuweb/
//! [`wgpu_glyph`]: https://github.com/hecrj/wgpu_glyph
#![deny(missing_docs)]
#![deny(missing_debug_implementations)]
#![deny(unused_results)]
#![deny(unsafe_code)]
#![forbid(rust_2018_idioms)]
#![cfg_attr(docsrs, feature(doc_cfg))]

pub mod settings;
pub mod triangle;
pub mod widget;
pub mod window;

mod backend;
mod quad;
mod text;

pub use iced_graphics::{
    Antialiasing, Color, Defaults, Error, Primitive, Viewport,
};
pub use wgpu;

pub use backend::Backend;
pub use settings::Settings;

#[doc(no_inline)]
pub use widget::*;

pub(crate) use iced_graphics::Transformation;

#[cfg(any(feature = "image", feature = "svg"))]
mod image;

/// A [`wgpu`] graphics renderer for [`iced`].
///
/// [`wgpu`]: https://github.com/gfx-rs/wgpu-rs
/// [`iced`]: https://github.com/hecrj/iced
pub type Renderer = iced_graphics::Renderer<Backend>;
