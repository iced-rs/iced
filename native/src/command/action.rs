use crate::clipboard;
use crate::window;

pub enum Action<T> {
    Future(iced_futures::BoxFuture<T>),
    Clipboard(clipboard::Action<T>),
    Window(window::Action),
}

impl<T> Action<T> {
    /// Applies a transformation to the result of a [`Command`].
    pub fn map<A>(self, f: impl Fn(T) -> A + 'static + Send + Sync) -> Action<A>
    where
        T: 'static,
    {
        use iced_futures::futures::FutureExt;

        match self {
            Self::Future(future) => Action::Future(Box::pin(future.map(f))),
            Self::Clipboard(action) => Action::Clipboard(action.map(f)),
            Self::Window(window) => Action::Window(window),
        }
    }
}
