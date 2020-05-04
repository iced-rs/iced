//! A renderer for [`iced_native`].
//!
//! ![`iced_shm` crate graph](https://github.com/hecrj/iced/blob/cae26cb7bc627f4a5b3bcf1cd023a0c552e8c65e/docs/graphs/wgpu.png?raw=true)
//!
//! A simple shared memory CPU renderer for the SCTK backend (Wayland)
//!
//! Can be used for development thanks to faster linking due to reduced code size (dependencies)
//!
//! Currently, `iced_shm` supports the following primitives:
//! - TODO: Text, which is rendered using [`framework`]. No shaping at all.
//! - TODO: Quads or rectangles, with rounded borders and a solid background color.
//! - TODO: Clip areas, useful to implement scrollables or hide overflowing content.
//! - TODO: Images and SVG, loaded from memory or the file system.
//!
//! [`framework`]: https://github.com/Matthias-Fauconneau/framework
#![feature(type_ascription,associated_type_defaults)]
#![deny(missing_docs)]
#![deny(missing_debug_implementations)]
#![deny(unused_results)]
#![deny(unsafe_code)]
#![cfg_attr(docsrs, feature(doc_cfg))]

pub mod defaults;
pub mod settings;
pub mod widget;
pub mod window;

mod primitive;
mod quad;
mod renderer;
mod target;
mod text;
mod transformation;
mod viewport;

pub use defaults::Defaults;
pub use primitive::Primitive;
pub use renderer::Renderer;
pub use settings::Settings;
pub use target::Target;
pub use viewport::Viewport;

#[doc(no_inline)]
pub use widget::*;

pub(crate) use quad::Quad;
pub(crate) use transformation::Transformation;

#[cfg(any(feature = "image", feature = "svg"))]
mod image;
