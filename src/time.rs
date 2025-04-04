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
