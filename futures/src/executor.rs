//! Choose your preferred executor to power a runtime.
mod null;

#[cfg(feature = "thread-pool")]
mod thread_pool;

#[cfg(feature = "thread-pool")]
pub use thread_pool::ThreadPool;

#[cfg(feature = "tokio")]
mod tokio;

#[cfg(feature = "async-std")]
mod async_std;

pub use null::Null;

#[cfg(feature = "tokio")]
pub use self::tokio::Tokio;

#[cfg(feature = "async-std")]
pub use self::async_std::AsyncStd;

use futures::Future;

pub trait Executor: Sized {
    fn new() -> Result<Self, futures::io::Error>
    where
        Self: Sized;

    fn spawn(&self, future: impl Future<Output = ()> + Send + 'static);

    fn enter<R>(&self, f: impl FnOnce() -> R) -> R {
        f()
    }
}
