//! Backends that are only available in native platforms: Windows, macOS, or Linux.
#[cfg(feature = "tokio")]
pub mod tokio;

#[cfg(feature = "smol")]
pub mod smol;

#[cfg(feature = "thread-pool")]
pub mod thread_pool;
