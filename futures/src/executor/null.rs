use crate::Executor;

use futures::Future;

pub struct Null;

impl Executor for Null {
    fn new() -> Result<Self, futures::io::Error> {
        Ok(Self)
    }

    fn spawn(&self, _future: impl Future<Output = ()> + Send + 'static) {}
}
