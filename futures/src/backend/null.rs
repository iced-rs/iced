//! A backend that does nothing!
use futures::Future;

/// An executor that drops all the futures, instead of spawning them.
#[derive(Debug)]
pub struct Executor;

impl crate::Executor for Executor {
    fn new() -> Result<Self, futures::io::Error> {
        Ok(Self)
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn spawn(&self, _future: impl Future<Output = ()> + Send + 'static) {}

    #[cfg(target_arch = "wasm32")]
    fn spawn(&self, _future: impl Future<Output = ()> + 'static) {}
}

pub mod time {
    //! Listen and react to time.
}
