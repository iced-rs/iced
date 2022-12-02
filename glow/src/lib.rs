//! A [`glow`] renderer for [`iced_native`].
//!
//! ![The native path of the Iced ecosystem](https://github.com/iced-rs/iced/blob/0525d76ff94e828b7b21634fa94a747022001c83/docs/graphs/native.png?raw=true)
//!
//! [`glow`]: https://github.com/grovesNL/glow
//! [`iced_native`]: https://github.com/iced-rs/iced/tree/0.5/native
#![doc(
    html_logo_url = "https://raw.githubusercontent.com/iced-rs/iced/9ab6923e943f784985e9ef9ca28b10278297225d/docs/logo.svg"
)]
#![deny(
    missing_debug_implementations,
    missing_docs,
    unused_results,
    clippy::extra_unused_lifetimes,
    clippy::from_over_into,
    clippy::needless_borrow,
    clippy::new_without_default,
    clippy::useless_conversion
)]
#![forbid(rust_2018_idioms)]
#![allow(clippy::inherent_to_string, clippy::type_complexity)]
#![cfg_attr(docsrs, feature(doc_cfg))]

pub use glow;

mod backend;
#[cfg(any(feature = "image", feature = "svg"))]
mod image;
mod program;
mod quad;
mod text;
mod triangle;

pub mod settings;
pub mod window;

pub use backend::Backend;
pub use settings::Settings;

pub(crate) use iced_graphics::Transformation;

pub use iced_graphics::{Error, Viewport};
pub use iced_native::Theme;

pub use iced_native::alignment;
pub use iced_native::{Alignment, Background, Color, Command, Length, Vector};

/// A [`glow`] graphics renderer for [`iced`].
///
/// [`glow`]: https://github.com/grovesNL/glow
/// [`iced`]: https://github.com/iced-rs/iced
pub type Renderer<Theme = iced_native::Theme> =
    iced_graphics::Renderer<Backend, Theme>;
