//! A windowing shell for Iced, on top of [`winit`].
//!
//! ![The native path of the Iced ecosystem](https://github.com/iced-rs/iced/blob/0525d76ff94e828b7b21634fa94a747022001c83/docs/graphs/native.png?raw=true)
//!
//! `iced_winit` offers some convenient abstractions on top of [`iced_runtime`]
//! to quickstart development when using [`winit`].
//!
//! It exposes a renderer-agnostic [`Application`] trait that can be implemented
//! and then run with a simple call. The use of this trait is optional.
//!
//! Additionally, a [`conversion`] module is available for users that decide to
//! implement a custom event loop.
//!
//! [`iced_runtime`]: https://github.com/iced-rs/iced/tree/0.10/runtime
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
    clippy::useless_conversion,
    unsafe_code
)]
#![forbid(rust_2018_idioms)]
#![allow(clippy::inherent_to_string, clippy::type_complexity)]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]
pub use iced_graphics as graphics;
pub use iced_runtime as runtime;
pub use iced_runtime::core;
pub use iced_runtime::futures;
pub use iced_style as style;
pub use winit;

#[cfg(feature = "application")]
pub mod application;
pub mod clipboard;
pub mod conversion;
pub mod settings;

#[cfg(feature = "system")]
pub mod system;

mod error;
mod position;
mod proxy;

#[cfg(feature = "application")]
pub use application::Application;
#[cfg(feature = "trace")]
pub use application::Profiler;
pub use clipboard::Clipboard;
pub use error::Error;
pub use position::Position;
pub use proxy::Proxy;
pub use settings::Settings;

pub use iced_graphics::Viewport;
