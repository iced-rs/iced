//! Asynchronous tasks for GUI programming, inspired by Elm.
//!
//! ![The foundations of the Iced ecosystem](https://github.com/iced-rs/iced/blob/0525d76ff94e828b7b21634fa94a747022001c83/docs/graphs/foundations.png?raw=true)
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
#![forbid(unsafe_code, rust_2018_idioms)]
#![allow(clippy::inherent_to_string, clippy::type_complexity)]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]
pub use futures;
pub use iced_core as core;

mod maybe_send;
mod runtime;

pub mod backend;
pub mod executor;
pub mod subscription;

pub use executor::Executor;
pub use maybe_send::MaybeSend;
pub use platform::*;
pub use runtime::Runtime;
pub use subscription::Subscription;

#[cfg(not(target_arch = "wasm32"))]
mod platform {
    /// A boxed static future.
    ///
    /// - On native platforms, it needs a `Send` requirement.
    /// - On the Web platform, it does not need a `Send` requirement.
    pub type BoxFuture<T> = futures::future::BoxFuture<'static, T>;

    /// A boxed static stream.
    ///
    /// - On native platforms, it needs a `Send` requirement.
    /// - On the Web platform, it does not need a `Send` requirement.
    pub type BoxStream<T> = futures::stream::BoxStream<'static, T>;

    /// Boxes a stream.
    ///
    /// - On native platforms, it needs a `Send` requirement.
    /// - On the Web platform, it does not need a `Send` requirement.
    pub fn boxed_stream<T, S>(stream: S) -> BoxStream<T>
    where
        S: futures::Stream<Item = T> + Send + 'static,
    {
        futures::stream::StreamExt::boxed(stream)
    }
}

#[cfg(target_arch = "wasm32")]
mod platform {
    /// A boxed static future.
    ///
    /// - On native platforms, it needs a `Send` requirement.
    /// - On the Web platform, it does not need a `Send` requirement.
    pub type BoxFuture<T> = futures::future::LocalBoxFuture<'static, T>;

    /// A boxed static stream.
    ///
    /// - On native platforms, it needs a `Send` requirement.
    /// - On the Web platform, it does not need a `Send` requirement.
    pub type BoxStream<T> = futures::stream::LocalBoxStream<'static, T>;

    /// Boxes a stream.
    ///
    /// - On native platforms, it needs a `Send` requirement.
    /// - On the Web platform, it does not need a `Send` requirement.
    pub fn boxed_stream<T, S>(stream: S) -> BoxStream<T>
    where
        S: futures::Stream<Item = T> + 'static,
    {
        futures::stream::StreamExt::boxed_local(stream)
    }
}
