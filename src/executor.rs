//! Choose your preferred executor to power your application.
pub use crate::common::{executor::Null, Executor};

pub use platform::Default;

#[cfg(not(target_arch = "wasm32"))]
mod platform {
    use iced_winit::{executor::ThreadPool, futures, Executor};

    /// A default cross-platform executor.
    ///
    /// - On native platforms, it will use a `iced_futures::executor::ThreadPool`.
    /// - On the Web, it will use `iced_futures::executor::WasmBindgen`.
    #[derive(Debug)]
    pub struct Default(ThreadPool);

    impl Executor for Default {
        fn new() -> Result<Self, futures::io::Error> {
            Ok(Default(ThreadPool::new()?))
        }

        fn spawn(
            &self,
            future: impl futures::Future<Output = ()> + Send + 'static,
        ) {
            self.0.spawn(future);
        }
    }
}

#[cfg(target_arch = "wasm32")]
mod platform {
    use iced_web::{executor::WasmBindgen, futures, Executor};

    /// A default cross-platform executor.
    ///
    /// - On native platforms, it will use a `iced_futures::executor::ThreadPool`.
    /// - On the Web, it will use `iced_futures::executor::WasmBindgen`.
    #[derive(Debug)]
    pub struct Default(WasmBindgen);

    impl Executor for Default {
        fn new() -> Result<Self, futures::io::Error> {
            Ok(Default(WasmBindgen::new()?))
        }

        fn spawn(
            &self,
            future: impl futures::Future<Output = ()> + Send + 'static,
        ) {
            self.0.spawn(future);
        }
    }
}
