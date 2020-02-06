use crate::Executor;

use futures::Future;

/// An executor that drops all the futures, instead of spawning them.
#[derive(Debug)]
pub struct Null;

impl Executor for Null {
    fn new() -> Result<Self, futures::io::Error> {
        Ok(Self)
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn spawn(&self, _future: impl Future<Output = ()> + Send + 'static) {}

    #[cfg(target_arch = "wasm32")]
    fn spawn(&self, _future: impl Future<Output = ()> + 'static) {}
}
