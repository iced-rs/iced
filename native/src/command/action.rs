use crate::clipboard;
use crate::system;
use crate::widget;
use crate::window;

use iced_futures::MaybeSend;

use std::fmt;

/// An action that a [`Command`] can perform.
///
/// [`Command`]: crate::Command
pub enum Action<T> {
    /// Run a [`Future`] to completion.
    ///
    /// [`Future`]: iced_futures::BoxFuture
    Future(iced_futures::BoxFuture<T>),

    /// Run a clipboard action.
    Clipboard(clipboard::Action<T>),

    /// Run a window action.
    Window(window::Action<T>),

    /// Run a system action.
    System(system::Action<T>),

    /// Run a widget action.
    Widget(widget::Action<T>),
}

impl<T> Action<T> {
    /// Applies a transformation to the result of a [`Command`].
    ///
    /// [`Command`]: crate::Command
    pub fn map<A>(
        self,
        f: impl Fn(T) -> A + 'static + MaybeSend + Sync,
    ) -> Action<A>
    where
        A: 'static,
        T: 'static,
    {
        use iced_futures::futures::FutureExt;

        match self {
            Self::Future(future) => Action::Future(Box::pin(future.map(f))),
            Self::Clipboard(action) => Action::Clipboard(action.map(f)),
            Self::Window(window) => Action::Window(window.map(f)),
            Self::System(system) => Action::System(system.map(f)),
            Self::Widget(widget) => Action::Widget(widget.map(f)),
        }
    }
}

impl<T> fmt::Debug for Action<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Future(_) => write!(f, "Action::Future"),
            Self::Clipboard(action) => {
                write!(f, "Action::Clipboard({action:?})")
            }
            Self::Window(action) => write!(f, "Action::Window({action:?})"),
            Self::System(action) => write!(f, "Action::System({action:?})"),
            Self::Widget(_action) => write!(f, "Action::Widget"),
        }
    }
}
