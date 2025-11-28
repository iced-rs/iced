//! Listen and react to time.
pub use crate::core::time::*;

#[allow(unused_imports)]
#[cfg_attr(
    docsrs,
    doc(cfg(any(
        feature = "tokio",
        feature = "smol",
        target_arch = "wasm32"
    )))
)]
pub use iced_futures::backend::default::time::*;

use crate::Task;

/// Returns a [`Task`] that produces the current [`Instant`]
/// by calling [`Instant::now`].
///
/// While you can call [`Instant::now`] directly in your application;
/// that renders your application "impure" (i.e. no referential transparency).
///
/// You may care about purity if you want to leverage the `time-travel`
/// feature properly.
pub fn now() -> Task<Instant> {
    Task::future(async { Instant::now() })
}
