//! A bunch of backend-agnostic types that can be leveraged to build a renderer
//! for [`iced`].
//!
//! ![The native path of the Iced ecosystem](https://github.com/iced-rs/iced/blob/0525d76ff94e828b7b21634fa94a747022001c83/docs/graphs/native.png?raw=true)
//!
//! [`iced`]: https://github.com/iced-rs/iced
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
mod antialiasing;
mod error;
mod primitive;
mod transformation;
mod viewport;

pub mod backend;
pub mod color;
pub mod compositor;
pub mod damage;
pub mod gradient;
pub mod mesh;
pub mod renderer;

#[cfg(feature = "geometry")]
pub mod geometry;

#[cfg(feature = "image")]
pub mod image;

pub use antialiasing::Antialiasing;
pub use backend::Backend;
pub use compositor::Compositor;
pub use damage::Damage;
pub use error::Error;
pub use gradient::Gradient;
pub use mesh::Mesh;
pub use primitive::Primitive;
pub use renderer::Renderer;
pub use transformation::Transformation;
pub use viewport::Viewport;

pub use iced_core as core;
