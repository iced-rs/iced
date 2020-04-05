use crate::Executor;

use futures::Future;

/// An `async-std` runtime.
#[cfg_attr(docsrs, doc(cfg(feature = "async-std")))]
#[derive(Debug)]
pub struct AsyncStd;

impl Executor for AsyncStd {
    fn new() -> Result<Self, futures::io::Error> {
        Ok(Self)
    }

    fn spawn(&self, future: impl Future<Output = ()> + Send + 'static) {
        let _ = async_std::task::spawn(future);
    }
}
