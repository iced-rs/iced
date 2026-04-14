//! A `ThreadPool` backend.

/// A thread pool executor for futures.
pub type Executor = futures::executor::ThreadPool;

impl crate::Executor for Executor {
    fn new() -> Result<Self, futures::io::Error> {
        let pool_size = std::env::var("ICED_THREAD_POOL_SIZE")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or_else(|| {
                std::thread::available_parallelism()
                    .map(|n| n.get().min(4))
                    .unwrap_or(4)
            });

        futures::executor::ThreadPoolBuilder::new()
            .pool_size(pool_size)
            .create()
    }

    fn spawn(&self, future: impl Future<Output = ()> + Send + 'static) {
        self.spawn_ok(future);
    }

    fn block_on<T>(&self, future: impl Future<Output = T>) -> T {
        futures::executor::block_on(future)
    }
}

pub mod time {
    //! Listen and react to time.
}
