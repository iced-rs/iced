//! A bunch of backend-agnostic types that can be leveraged to build a renderer
//! for [`iced`].
//!
//! ![The native path of the Iced ecosystem](https://github.com/iced-rs/iced/blob/0525d76ff94e828b7b21634fa94a747022001c83/docs/graphs/native.png?raw=true)
//!
//! [`iced`]: https://github.com/iced-rs/iced
#![doc(
    html_logo_url = "https://raw.githubusercontent.com/iced-rs/iced/9ab6923e943f784985e9ef9ca28b10278297225d/docs/logo.svg"
)]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]
mod antialiasing;
mod settings;
mod viewport;

pub mod cache;
pub mod color;
pub mod compositor;
pub mod damage;
pub mod error;
pub mod gradient;
pub mod image;
pub mod layer;
pub mod mesh;
pub mod text;

#[cfg(feature = "geometry")]
pub mod geometry;

pub use antialiasing::Antialiasing;
pub use cache::Cache;
pub use compositor::Compositor;
pub use error::Error;
pub use gradient::Gradient;
pub use image::Image;
pub use layer::Layer;
pub use mesh::Mesh;
pub use settings::Settings;
pub use text::Text;
pub use viewport::Viewport;

pub use iced_core as core;
pub use iced_futures as futures;
