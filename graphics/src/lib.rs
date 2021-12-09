//! A bunch of backend-agnostic types that can be leveraged to build a renderer
//! for [`iced`].
//!
//! ![The native path of the Iced ecosystem](https://github.com/hecrj/iced/blob/0525d76ff94e828b7b21634fa94a747022001c83/docs/graphs/native.png?raw=true)
//!
//! [`iced`]: https://github.com/hecrj/iced
#![doc(
    html_logo_url = "https://raw.githubusercontent.com/iced-rs/iced/9ab6923e943f784985e9ef9ca28b10278297225d/docs/logo.svg"
)]
#![deny(missing_docs)]
#![deny(missing_debug_implementations)]
#![deny(unused_results)]
#![deny(unsafe_code)]
#![forbid(rust_2018_idioms)]
#![cfg_attr(docsrs, feature(doc_cfg))]
mod antialiasing;
mod error;
mod primitive;
mod transformation;
mod viewport;

pub mod backend;
pub mod font;
pub mod layer;
pub mod overlay;
pub mod renderer;
pub mod triangle;
pub mod widget;
pub mod window;

#[doc(no_inline)]
pub use widget::*;

pub use antialiasing::Antialiasing;
pub use backend::Backend;
pub use error::Error;
pub use layer::Layer;
pub use primitive::Primitive;
pub use renderer::Renderer;
pub use transformation::Transformation;
pub use viewport::Viewport;

pub use iced_native::alignment;
pub use iced_native::{
    Alignment, Background, Color, Font, Point, Rectangle, Size, Vector,
};
