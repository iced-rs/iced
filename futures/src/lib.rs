//! Asynchronous tasks for GUI programming, inspired by Elm.
//!
//! ![The foundations of the Iced ecosystem](https://github.com/iced-rs/iced/blob/0525d76ff94e828b7b21634fa94a747022001c83/docs/graphs/foundations.png?raw=true)
#![doc(
    html_logo_url = "https://raw.githubusercontent.com/iced-rs/iced/9ab6923e943f784985e9ef9ca28b10278297225d/docs/logo.svg"
)]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]
pub use futures;
pub use iced_core as core;

mod maybe;
mod runtime;

pub mod backend;
pub mod event;
pub mod executor;
pub mod keyboard;
pub mod stream;
pub mod subscription;

pub use executor::Executor;
pub use maybe::{MaybeSend, MaybeSync};
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
