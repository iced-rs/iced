//! Backends that are only available in native platforms: Windows, macOS, or Linux.
#[cfg_attr(docsrs, doc(cfg(feature = "tokio",)))]
#[cfg(feature = "tokio")]
pub mod tokio;

#[cfg_attr(docsrs, doc(cfg(feature = "async-std",)))]
#[cfg(feature = "async-std")]
pub mod async_std;

#[cfg_attr(docsrs, doc(cfg(feature = "smol",)))]
#[cfg(feature = "smol")]
pub mod smol;

#[cfg_attr(docsrs, doc(cfg(feature = "thread-pool",)))]
#[cfg(feature = "thread-pool")]
pub mod thread_pool;
