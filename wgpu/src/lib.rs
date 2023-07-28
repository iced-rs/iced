//! A [`wgpu`] renderer for [Iced].
//!
//! ![The native path of the Iced ecosystem](https://github.com/iced-rs/iced/blob/0525d76ff94e828b7b21634fa94a747022001c83/docs/graphs/native.png?raw=true)
//!
//! [`wgpu`] supports most modern graphics backends: Vulkan, Metal, DX11, and
//! DX12 (OpenGL and WebGL are still WIP). Additionally, it will support the
//! incoming [WebGPU API].
//!
//! Currently, `iced_wgpu` supports the following primitives:
//! - Text, which is rendered using [`glyphon`].
//! - Quads or rectangles, with rounded borders and a solid background color.
//! - Clip areas, useful to implement scrollables or hide overflowing content.
//! - Images and SVG, loaded from memory or the file system.
//! - Meshes of triangles, useful to draw geometry freely.
//!
//! [Iced]: https://github.com/iced-rs/iced
//! [`wgpu`]: https://github.com/gfx-rs/wgpu-rs
//! [WebGPU API]: https://gpuweb.github.io/gpuweb/
//! [`glyphon`]: https://github.com/grovesNL/glyphon
#![doc(
    html_logo_url = "https://raw.githubusercontent.com/iced-rs/iced/9ab6923e943f784985e9ef9ca28b10278297225d/docs/logo.svg"
)]
#![deny(
    missing_debug_implementations,
    missing_docs,
    unsafe_code,
    unused_results,
    clippy::extra_unused_lifetimes,
    clippy::from_over_into,
    clippy::needless_borrow,
    clippy::new_without_default,
    clippy::useless_conversion
)]
#![forbid(rust_2018_idioms)]
#![allow(clippy::inherent_to_string, clippy::type_complexity)]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]
pub mod layer;
pub mod primitive;
pub mod settings;
pub mod window;

#[cfg(feature = "geometry")]
pub mod geometry;

mod backend;
mod buffer;
mod color;
mod quad;
mod text;
mod triangle;

use buffer::Buffer;

pub use iced_graphics as graphics;
pub use iced_graphics::core;

pub use wgpu;

pub use backend::Backend;
pub use layer::Layer;
pub use primitive::Primitive;
pub use settings::Settings;

#[cfg(any(feature = "image", feature = "svg"))]
mod image;

/// A [`wgpu`] graphics renderer for [`iced`].
///
/// [`wgpu`]: https://github.com/gfx-rs/wgpu-rs
/// [`iced`]: https://github.com/iced-rs/iced
pub type Renderer<Theme> = iced_graphics::Renderer<Backend, Theme>;
