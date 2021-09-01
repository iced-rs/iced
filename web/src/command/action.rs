pub enum Action<T> {
    Future(iced_futures::BoxFuture<T>),
}

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
