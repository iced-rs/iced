use crate::Executor;

use futures::Future;

/// An executor that drops all the futures, instead of spawning them.
#[derive(Debug)]
pub struct Null;

impl Executor for Null {
    fn new() -> Result<Self, futures::io::Error> {
        Ok(Self)
    }

    fn spawn(&self, _future: impl Future<Output = ()> + Send + 'static) {}
}
