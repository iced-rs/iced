//! Keep track of time, both in native and web platforms!

#[cfg(target_arch = "wasm32")]
pub use instant::Instant;

#[cfg(target_arch = "wasm32")]
pub use instant::Duration;

#[cfg(not(target_arch = "wasm32"))]
pub use std::time::Instant;

#[cfg(not(target_arch = "wasm32"))]
pub use std::time::Duration;
