//! A windowing shell for [`iced`], on top of [`glutin`].
//!
//! ![The native path of the Iced ecosystem](https://github.com/iced-rs/iced/blob/0525d76ff94e828b7b21634fa94a747022001c83/docs/graphs/native.png?raw=true)
//!
//! [`iced`]: https://github.com/iced-rs/iced
//! [`glutin`]: https://github.com/rust-windowing/glutin
#![doc(
    html_logo_url = "https://raw.githubusercontent.com/iced-rs/iced/9ab6923e943f784985e9ef9ca28b10278297225d/docs/logo.svg"
)]
#![deny(
    missing_docs,
    missing_debug_implementations,
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
#![cfg_attr(docsrs, feature(doc_cfg))]

pub use glutin;

#[doc(no_inline)]
pub use iced_winit::*;

pub mod application;
mod compositor;

#[doc(no_inline)]
pub use application::Application;
pub use compositor::{Compositor, Settings};
