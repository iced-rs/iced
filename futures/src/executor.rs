//! Choose your preferred executor to power a runtime.
mod null;

#[cfg(all(not(target_arch = "wasm32"), feature = "thread-pool"))]
mod thread_pool;

#[cfg(all(not(target_arch = "wasm32"), feature = "tokio"))]
mod tokio;

#[cfg(all(not(target_arch = "wasm32"), feature = "tokio_old"))]
mod tokio_old;

#[cfg(all(not(target_arch = "wasm32"), feature = "async-std"))]
mod async_std;

#[cfg(all(not(target_arch = "wasm32"), feature = "smol"))]
mod smol;

#[cfg(target_arch = "wasm32")]
mod wasm_bindgen;

pub use null::Null;

#[cfg(all(not(target_arch = "wasm32"), feature = "thread-pool"))]
pub use thread_pool::ThreadPool;

#[cfg(all(not(target_arch = "wasm32"), feature = "tokio"))]
pub use self::tokio::Tokio;

#[cfg(all(not(target_arch = "wasm32"), feature = "tokio_old"))]
pub use self::tokio_old::TokioOld;

#[cfg(all(not(target_arch = "wasm32"), feature = "async-std"))]
pub use self::async_std::AsyncStd;

#[cfg(all(not(target_arch = "wasm32"), feature = "smol"))]
pub use self::smol::Smol;

#[cfg(target_arch = "wasm32")]
pub use wasm_bindgen::WasmBindgen;

use futures::Future;

/// A type that can run futures.
pub trait Executor: Sized {
    /// Creates a new [`Executor`].
    fn new() -> Result<Self, futures::io::Error>
    where
        Self: Sized;

    /// Spawns a future in the [`Executor`].
    #[cfg(not(target_arch = "wasm32"))]
    fn spawn(&self, future: impl Future<Output = ()> + Send + 'static);

    /// Spawns a local future in the [`Executor`].
    #[cfg(target_arch = "wasm32")]
    fn spawn(&self, future: impl Future<Output = ()> + 'static);

    /// Runs the given closure inside the [`Executor`].
    ///
    /// Some executors, like `tokio`, require some global state to be in place
    /// before creating futures. This method can be leveraged to set up this
    /// global state, call a function, restore the state, and obtain the result
    /// of the call.
    fn enter<R>(&self, f: impl FnOnce() -> R) -> R {
        f()
    }
}
