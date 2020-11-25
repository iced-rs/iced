use crate::Executor;

use futures::Future;

/// An old `tokio` runtime.
#[cfg_attr(docsrs, doc(cfg(feature = "tokio_old")))]
pub type TokioOld = tokio_old::runtime::Runtime;

impl Executor for TokioOld {
    fn new() -> Result<Self, futures::io::Error> {
        tokio_old::runtime::Runtime::new()
    }

    fn spawn(&self, future: impl Future<Output = ()> + Send + 'static) {
        let _ = tokio_old::runtime::Runtime::spawn(self, future);
    }

    fn enter<R>(&self, f: impl FnOnce() -> R) -> R {
        tokio_old::runtime::Runtime::enter(self, f)
    }
}
