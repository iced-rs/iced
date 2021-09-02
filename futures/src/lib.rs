//! Asynchronous tasks for GUI programming, inspired by Elm.
//!
//! ![The foundations of the Iced ecosystem](https://github.com/hecrj/iced/blob/0525d76ff94e828b7b21634fa94a747022001c83/docs/graphs/foundations.png?raw=true)
#![deny(missing_docs)]
#![deny(missing_debug_implementations)]
#![deny(unused_results)]
#![forbid(unsafe_code)]
#![forbid(rust_2018_idioms)]
#![cfg_attr(docsrs, feature(doc_cfg))]

pub use futures;

mod command;
mod runtime;

pub mod executor;
pub mod subscription;

#[cfg(all(
    any(
        feature = "tokio",
        feature = "tokio_old",
        feature = "async-std",
        feature = "smol"
    ),
    not(target_arch = "wasm32")
))]
#[cfg_attr(
    docsrs,
    doc(cfg(any(
        feature = "tokio",
        feature = "async-std",
        feature = "smol"
    )))
)]
pub mod time;

pub use command::Command;
pub use executor::Executor;
pub use runtime::Runtime;
pub use subscription::Subscription;

/// A boxed static future.
///
/// - On native platforms, it needs a `Send` requirement.
/// - On the Web platform, it does not need a `Send` requirement.
#[cfg(not(target_arch = "wasm32"))]
pub type BoxFuture<T> = futures::future::BoxFuture<'static, T>;

/// A boxed static future.
///
/// - On native platforms, it needs a `Send` requirement.
/// - On the Web platform, it does not need a `Send` requirement.
#[cfg(target_arch = "wasm32")]
pub type BoxFuture<T> = futures::future::LocalBoxFuture<'static, T>;

/// A boxed static stream.
///
/// - On native platforms, it needs a `Send` requirement.
/// - On the Web platform, it does not need a `Send` requirement.
#[cfg(not(target_arch = "wasm32"))]
pub type BoxStream<T> = futures::stream::BoxStream<'static, T>;

/// A boxed static stream.
///
/// - On native platforms, it needs a `Send` requirement.
/// - On the Web platform, it does not need a `Send` requirement.
#[cfg(target_arch = "wasm32")]
pub type BoxStream<T> = futures::stream::LocalBoxStream<'static, T>;
