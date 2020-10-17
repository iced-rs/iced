//! Choose your preferred executor to power your application.
pub use crate::runtime::Executor;

pub use platform::Default;

#[cfg(not(target_arch = "wasm32"))]
mod platform {
    use iced_futures::{executor, futures};

    #[cfg(feature = "tokio")]
    type Executor = executor::Tokio;

    #[cfg(all(not(feature = "tokio"), feature = "async-std"))]
    type Executor = executor::AsyncStd;

    #[cfg(not(any(feature = "tokio", feature = "async-std")))]
    type Executor = executor::ThreadPool;

    /// A default cross-platform executor.
    ///
    /// - On native platforms, it will use:
    ///   - `iced_futures::executor::Tokio` when the `tokio` feature is enabled.
    ///   - `iced_futures::executor::AsyncStd` when the `async-std` feature is
    ///     enabled.
    ///   - `iced_futures::executor::ThreadPool` otherwise.
    /// - On the Web, it will use `iced_futures::executor::WasmBindgen`.
    #[derive(Debug)]
    pub struct Default(Executor);

    impl super::Executor for Default {
        fn new() -> Result<Self, futures::io::Error> {
            Ok(Default(Executor::new()?))
        }

        fn spawn(
            &self,
            future: impl futures::Future<Output = ()> + Send + 'static,
        ) {
            let _ = self.0.spawn(future);
        }

        fn enter<R>(&self, f: impl FnOnce() -> R) -> R {
            self.0.enter(f)
        }
    }
}

#[cfg(target_arch = "wasm32")]
mod platform {
    use iced_futures::{executor::WasmBindgen, futures, Executor};

    /// A default cross-platform executor.
    ///
    /// - On native platforms, it will use:
    ///   - `iced_futures::executor::Tokio` when the `tokio` feature is enabled.
    ///   - `iced_futures::executor::AsyncStd` when the `async-std` feature is
    ///     enabled.
    ///   - `iced_futures::executor::ThreadPool` otherwise.
    /// - On the Web, it will use `iced_futures::executor::WasmBindgen`.
    #[derive(Debug)]
    pub struct Default(WasmBindgen);

    impl Executor for Default {
        fn new() -> Result<Self, futures::io::Error> {
            Ok(Default(WasmBindgen::new()?))
        }

        fn spawn(&self, future: impl futures::Future<Output = ()> + 'static) {
            self.0.spawn(future);
        }

        fn enter<R>(&self, f: impl FnOnce() -> R) -> R {
            self.0.enter(f)
        }
    }
}
