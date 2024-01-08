//! Listen and react to time.
pub use iced_core::time::{Duration, Instant};

#[allow(unused_imports)]
#[cfg_attr(
    docsrs,
    doc(cfg(any(
        feature = "tokio",
        feature = "async-std",
        feature = "smol",
        target_arch = "wasm32"
    )))
)]
pub use iced_futures::backend::default::time::*;
