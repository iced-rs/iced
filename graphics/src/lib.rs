//! A bunch of backend-agnostic types that can be leveraged to build a renderer
//! for [`iced`].
//!
//! [`iced`]: https://github.com/hecrj/iced
#![deny(missing_docs)]
#![deny(missing_debug_implementations)]
#![deny(unused_results)]
#![deny(unsafe_code)]
#![forbid(rust_2018_idioms)]
#![cfg_attr(docsrs, feature(doc_cfg))]
mod antialiasing;
mod error;
mod primitive;
mod renderer;
mod transformation;
mod viewport;

pub mod backend;
pub mod defaults;
pub mod font;
pub mod layer;
pub mod overlay;
pub mod triangle;
pub mod widget;
pub mod window;

#[doc(no_inline)]
pub use widget::*;

pub use antialiasing::Antialiasing;
pub use backend::Backend;
pub use defaults::Defaults;
pub use error::Error;
pub use layer::Layer;
pub use primitive::Primitive;
pub use renderer::Renderer;
pub use transformation::Transformation;
pub use viewport::Viewport;

pub use iced_native::{
    Background, Color, Font, HorizontalAlignment, Point, Rectangle, Size,
    Vector, VerticalAlignment,
};
