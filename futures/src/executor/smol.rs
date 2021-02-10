use crate::Executor;

use futures::Future;

/// A `smol` runtime.
#[cfg_attr(docsrs, doc(cfg(feature = "smol")))]
#[derive(Debug)]
pub struct Smol;

impl Executor for Smol {
    fn new() -> Result<Self, futures::io::Error> {
        Ok(Self)
    }

    fn spawn(&self, future: impl Future<Output = ()> + Send + 'static) {
        smol::spawn(future).detach();
    }
}
