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
//! - Images, lazily loaded from the filesystem.
//! - Clip areas, useful to implement scrollables or hide overflowing content.
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
#![deny(rust_2018_idioms)]
mod image;
mod primitive;
mod quad;
mod renderer;
mod style;
mod text;
mod transformation;

pub(crate) use crate::image::Image;
pub(crate) use quad::Quad;
pub(crate) use transformation::Transformation;

pub use primitive::Primitive;
pub use renderer::{Renderer, Target};
pub use style::*;
