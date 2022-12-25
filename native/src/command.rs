//! Run asynchronous command.

use std::fmt;

use iced_futures::MaybeSend;

use crate::clipboard;
use crate::system;
use crate::widget;
use crate::window;

/// An asynchronous command to be performed by some runtime.
pub enum Command<T> {
    /// Run a clipboard action.
    Clipboard(clipboard::Action<T>),

    /// Run a window action.
    Window(window::Action<T>),

    /// Run a system action.
    System(system::Action<T>),

    /// Run a widget action.
    Widget(widget::Action<T>),
}

impl<T> Command<T> {
    /// Creates a [`Command`] that performs a [`widget::Operation`].
    pub fn widget(operation: impl widget::Operation<T> + 'static) -> Self {
        Self::Widget(widget::Action::new(operation))
    }

    /// Applies a transformation to the result of a [`Command`].
    ///
    /// [`Command`]: crate::Command
    pub fn map<A>(
        self,
        f: impl Fn(T) -> A + 'static + MaybeSend + Sync,
    ) -> Command<A>
    where
        A: 'static,
        T: 'static,
    {
        match self {
            Self::Clipboard(action) => Command::Clipboard(action.map(f)),
            Self::Window(window) => Command::Window(window.map(f)),
            Self::System(system) => Command::System(system.map(f)),
            Self::Widget(widget) => Command::Widget(widget.map(f)),
        }
    }
}

impl<T> fmt::Debug for Command<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Clipboard(action) => {
                write!(f, "Action::Clipboard({:?})", action)
            }
            Self::Window(action) => write!(f, "Action::Window({:?})", action),
            Self::System(action) => write!(f, "Action::System({:?})", action),
            Self::Widget(_action) => write!(f, "Action::Widget"),
        }
    }
}
