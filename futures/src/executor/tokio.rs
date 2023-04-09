use crate::Executor;

use futures::Future;

/// A `tokio` runtime.
#[cfg_attr(docsrs, doc(cfg(feature = "tokio")))]
pub type Tokio = tokio::runtime::Runtime;

impl Executor for Tokio {
    fn new() -> Result<Self, futures::io::Error> {
        tokio::runtime::Runtime::new()
    }

    fn spawn(&self, future: impl Future<Output = ()> + Send + 'static) {
        let _ = tokio::runtime::Runtime::spawn(self, future);
    }

    fn enter<R>(&self, f: impl FnOnce() -> R) -> R {
        let _guard = tokio::runtime::Runtime::enter(self);
        f()
    }
}
