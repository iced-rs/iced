//! A windowing shell for Iced, on top of [`winit`].
//!
//! ![The native path of the Iced ecosystem](https://github.com/iced-rs/iced/blob/0525d76ff94e828b7b21634fa94a747022001c83/docs/graphs/native.png?raw=true)
//!
//! `iced_winit` offers some convenient abstractions on top of [`iced_native`]
//! to quickstart development when using [`winit`].
//!
//! It exposes a renderer-agnostic [`Application`] trait that can be implemented
//! and then run with a simple call. The use of this trait is optional.
//!
//! Additionally, a [`conversion`] module is available for users that decide to
//! implement a custom event loop.
//!
//! [`iced_native`]: https://github.com/iced-rs/iced/tree/0.4/native
//! [`winit`]: https://github.com/rust-windowing/winit
//! [`conversion`]: crate::conversion
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
#![forbid(rust_2018_idioms, unsafe_code)]
#![allow(clippy::inherent_to_string, clippy::type_complexity)]
#![cfg_attr(docsrs, feature(doc_cfg))]

#[doc(no_inline)]
pub use iced_native::*;
pub use winit;

pub mod application;
pub mod clipboard;
pub mod conversion;
pub mod settings;
pub mod window;

#[cfg(feature = "system")]
pub mod system;

mod error;
mod mode;
mod position;
mod proxy;

pub use application::Application;
pub use clipboard::Clipboard;
pub use error::Error;
pub use mode::Mode;
pub use position::Position;
pub use proxy::Proxy;
pub use settings::Settings;

pub use iced_graphics::Viewport;
