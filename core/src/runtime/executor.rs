use futures::Future;

pub trait Executor {
    fn new() -> Self;

    fn spawn(&self, future: impl Future<Output = ()> + Send + 'static);

    fn enter<R>(&self, f: impl FnOnce() -> R) -> R {
        f()
    }
}
