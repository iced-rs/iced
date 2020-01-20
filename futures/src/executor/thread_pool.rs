use crate::Executor;

use futures::Future;

pub type ThreadPool = futures::executor::ThreadPool;

impl Executor for futures::executor::ThreadPool {
    fn new() -> Result<Self, futures::io::Error> {
        futures::executor::ThreadPool::new()
    }

    fn spawn(&self, future: impl Future<Output = ()> + Send + 'static) {
        self.spawn_ok(future);
    }
}
