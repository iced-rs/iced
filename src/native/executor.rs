//! Choose your preferred executor to power your application.
pub use iced_winit::{executor::Null, Executor};
use iced_winit::{executor::ThreadPool, futures};

/// The default cross-platform executor.
///
/// - On native platforms, it will use a `ThreadPool`.
/// - On the Web, it will use `wasm-bindgen-futures::spawn_local`.
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
