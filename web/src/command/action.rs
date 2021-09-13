pub enum Action<T> {
    Future(iced_futures::BoxFuture<T>),
}

use std::fmt;

impl<T> Action<T> {
    /// Applies a transformation to the result of a [`Command`].
    #[cfg(target_arch = "wasm32")]
    pub fn map<A>(self, f: impl Fn(T) -> A + 'static) -> Action<A>
    where
        T: 'static,
    {
        use iced_futures::futures::FutureExt;

        match self {
            Self::Future(future) => Action::Future(Box::pin(future.map(f))),
        }
    }
}

impl<T> fmt::Debug for Action<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Future(_) => write!(f, "Action::Future"),
        }
    }
}
