//! Choose your preferred executor to power a runtime.
use crate::MaybeSend;

/// A type that can run futures.
pub trait Executor: Sized {
    /// Creates a new [`Executor`].
    fn new() -> Result<Self, futures::io::Error>
    where
        Self: Sized;

    /// Spawns a future in the [`Executor`].
    fn spawn(&self, future: impl Future<Output = ()> + MaybeSend + 'static);

    /// Runs a future to completion in the current thread within the [`Executor`].
    #[cfg(not(target_arch = "wasm32"))]
    fn block_on<T>(&self, future: impl Future<Output = T>) -> T;

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
