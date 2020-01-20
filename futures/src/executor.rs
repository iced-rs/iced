//! Choose your preferred executor to power a runtime.
mod null;

#[cfg(feature = "thread-pool")]
mod thread_pool;

#[cfg(feature = "tokio")]
mod tokio;

#[cfg(feature = "async-std")]
mod async_std;

#[cfg(target_arch = "wasm32")]
mod wasm_bindgen;

pub use null::Null;

#[cfg(feature = "thread-pool")]
pub use thread_pool::ThreadPool;

#[cfg(feature = "tokio")]
pub use self::tokio::Tokio;

#[cfg(feature = "async-std")]
pub use self::async_std::AsyncStd;

#[cfg(target_arch = "wasm32")]
pub use wasm_bindgen::WasmBindgen;

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
