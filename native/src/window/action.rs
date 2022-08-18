use crate::window::Mode;

use iced_futures::MaybeSend;
use std::fmt;

/// An operation to be performed on some window.
pub enum Action<T> {
    /// Resize the window.
    Resize {
        /// The new logical width of the window
        width: u32,
        /// The new logical height of the window
        height: u32,
    },
    /// Move the window.
    Move {
        /// The new logical x location of the window
        x: i32,
        /// The new logical y location of the window
        y: i32,
    },
    /// Set the [`Mode`] of the window.
    SetMode(Mode),
    /// Fetch the current [`Mode`] of the window.
    FetchMode(Box<dyn FnOnce(Mode) -> T + 'static>),
}

impl<T> Action<T> {
    /// Maps the output of a window [`Action`] using the provided closure.
    pub fn map<A>(
        self,
        f: impl Fn(T) -> A + 'static + MaybeSend + Sync,
    ) -> Action<A>
    where
        T: 'static,
    {
        match self {
            Self::Resize { width, height } => Action::Resize { width, height },
            Self::Move { x, y } => Action::Move { x, y },
            Self::SetMode(mode) => Action::SetMode(mode),
            Self::FetchMode(o) => Action::FetchMode(Box::new(move |s| f(o(s)))),
        }
    }
}

impl<T> fmt::Debug for Action<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Resize { width, height } => write!(
                f,
                "Action::Resize {{ widget: {}, height: {} }}",
                width, height
            ),
            Self::Move { x, y } => {
                write!(f, "Action::Move {{ x: {}, y: {} }}", x, y)
            }
            Self::SetMode(mode) => write!(f, "Action::SetMode({:?})", mode),
            Self::FetchMode(_) => write!(f, "Action::FetchMode"),
        }
    }
}
