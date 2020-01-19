use futures::Future;

pub trait Executor {
    fn spawn(&self, future: impl Future<Output = ()> + Send + 'static);

    fn enter<R>(&self, f: impl FnOnce() -> R) -> R {
        f()
    }
}

#[cfg(feature = "thread-pool")]
impl Executor for futures::executor::ThreadPool {
    fn spawn(&self, future: impl Future<Output = ()> + Send + 'static) {
        self.spawn_ok(future);
    }
}

#[cfg(feature = "tokio")]
impl Executor for tokio::runtime::Runtime {
    fn spawn(&self, future: impl Future<Output = ()> + Send + 'static) {
        let _ = tokio::runtime::Runtime::spawn(self, future);
    }

    fn enter<R>(&self, f: impl FnOnce() -> R) -> R {
        tokio::runtime::Runtime::enter(self, f)
    }
}
