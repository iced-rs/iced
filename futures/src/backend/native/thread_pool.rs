//! A `ThreadPool` backend.
use futures::Future;

/// A thread pool executor for futures.
pub type Executor = futures::executor::ThreadPool;

impl crate::Executor for Executor {
    fn new() -> Result<Self, futures::io::Error> {
        futures::executor::ThreadPool::new()
    }

    fn spawn(&self, future: impl Future<Output = ()> + Send + 'static) {
        self.spawn_ok(future);
    }
}

pub mod time {
    //! Listen and react to time.
}
