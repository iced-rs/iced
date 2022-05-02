//! The underlying implementations of the `iced_futures` contract!
pub mod null;

#[cfg(not(target_arch = "wasm32"))]
pub mod native;

#[cfg(target_arch = "wasm32")]
pub mod wasm;

pub mod default;
