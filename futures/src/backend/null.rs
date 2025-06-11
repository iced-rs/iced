//! A backend that does nothing!
use crate::MaybeSend;

/// An executor that drops all the futures, instead of spawning them.
#[derive(Debug)]
pub struct Executor;

impl crate::Executor for Executor {
    fn new() -> Result<Self, futures::io::Error> {
        Ok(Self)
    }

    fn spawn(&self, _future: impl Future<Output = ()> + MaybeSend + 'static) {}

    #[cfg(not(target_arch = "wasm32"))]
    fn block_on<T>(&self, _future: impl Future<Output = T>) -> T {
        unimplemented!()
    }
}

pub mod time {
    //! Listen and react to time.
}
