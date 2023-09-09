#![forbid(rust_2018_idioms)]
#![deny(
    unsafe_code,
    unused_results,
    clippy::extra_unused_lifetimes,
    clippy::from_over_into,
    clippy::needless_borrow,
    clippy::new_without_default,
    clippy::useless_conversion,
    rustdoc::broken_intra_doc_links
)]
#![allow(clippy::inherent_to_string, clippy::type_complexity)]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]
pub mod window;

mod backend;
mod primitive;
mod settings;
mod text;

#[cfg(feature = "image")]
mod raster;

#[cfg(feature = "svg")]
mod vector;

#[cfg(feature = "geometry")]
pub mod geometry;

pub use iced_graphics as graphics;
pub use iced_graphics::core;

pub use backend::Backend;
pub use primitive::Primitive;
pub use settings::Settings;

/// A [`tiny-skia`] graphics renderer for [`iced`].
///
/// [`tiny-skia`]: https://github.com/RazrFalcon/tiny-skia
/// [`iced`]: https://github.com/iced-rs/iced
pub type Renderer<Theme> = iced_graphics::Renderer<Backend, Theme>;
