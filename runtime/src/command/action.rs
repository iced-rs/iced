use crate::clipboard;
use crate::core::widget;
use crate::font;
use crate::system;
use crate::window;

use iced_futures::MaybeSend;

use std::borrow::Cow;
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
    Widget(Box<dyn widget::Operation<T>>),

    /// Load a font from its bytes.
    LoadFont {
        /// The bytes of the font to load.
        bytes: Cow<'static, [u8]>,

        /// The message to produce when the font has been loaded.
        tagger: Box<dyn Fn(Result<(), font::Error>) -> T>,
    },

    /// Run a platform specific action
    PlatformSpecific(crate::command::platform_specific::Action<T>),
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
            Self::Widget(operation) => {
                Action::Widget(Box::new(widget::operation::map(operation, f)))
            }
            Self::LoadFont { bytes, tagger } => Action::LoadFont {
                bytes,
                tagger: Box::new(move |result| f(tagger(result))),
            },
            Self::PlatformSpecific(action) => {
                Action::PlatformSpecific(action.map(f))
            }
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
            Self::LoadFont { .. } => write!(f, "Action::LoadFont"),
            Self::PlatformSpecific(action) => {
                write!(f, "Action::PlatformSpecific({:?})", action)
            }
        }
    }
}
