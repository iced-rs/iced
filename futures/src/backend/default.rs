//! A default, cross-platform backend.
//!
//! - On native platforms, it will use:
//!   - `backend::native::tokio` when the `tokio` feature is enabled.
//!   - `backend::native::async-std` when the `async-std` feature is
//!     enabled.
//!   - `backend::native::smol` when the `smol` feature is enabled.
//!   - `backend::native::thread_pool` otherwise.
//!
//! - On Wasm, it will use `backend::wasm::wasm_bindgen`.
#[cfg(not(target_arch = "wasm32"))]
mod platform {
    #[cfg(feature = "tokio")]
    pub use crate::backend::native::tokio::*;

    #[cfg(all(feature = "async-std", not(feature = "tokio"),))]
    pub use crate::backend::native::async_std::*;

    #[cfg(all(
        feature = "smol",
        not(any(feature = "tokio", feature = "async-std")),
    ))]
    pub use crate::backend::native::smol::*;

    #[cfg(all(
        feature = "thread-pool",
        not(any(feature = "tokio", feature = "async-std", feature = "smol"))
    ))]
    pub use crate::backend::native::thread_pool::*;

    #[cfg(not(any(
        feature = "tokio",
        feature = "async-std",
        feature = "smol",
        feature = "thread-pool"
    )))]
    pub use crate::backend::null::*;
}

#[cfg(target_arch = "wasm32")]
mod platform {
    pub use crate::backend::wasm::wasm_bindgen::*;
}

pub use platform::*;
