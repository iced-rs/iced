use crate::window::{Icon, Mode};

use iced_futures::MaybeSend;
use std::fmt;

/// An operation to be performed on some window.
pub enum Action<T> {
    /// Closes the current window and exits the application.
    Close,
    /// Moves the window with the left mouse button until the button is
    /// released.
    ///
    /// There’s no guarantee that this will work unless the left mouse
    /// button was pressed immediately before this function is called.
    Drag,
    /// Resize the window.
    Resize {
        /// The new logical width of the window
        width: u32,
        /// The new logical height of the window
        height: u32,
    },
    /// Sets the window to maximized or back
    Maximize(bool),
    /// Set the window to minimized or back
    Minimize(bool),
    /// Move the window.
    ///
    /// Unsupported on Wayland.
    Move {
        /// The new logical x location of the window
        x: i32,
        /// The new logical y location of the window
        y: i32,
    },
    /// Set the [`Mode`] of the window.
    SetMode(Mode),
    /// Sets the window to maximized or back
    ToggleMaximize,
    /// Sets the window icon.
    ///
    /// On Windows and X11, this is typically the small icon in the top-left
    /// corner of the titlebar.
    ///
    /// ## Platform-specific
    ///
    /// - **Web / Wayland / macOS:** Unsupported.
    ///
    /// - **Windows:** Sets `ICON_SMALL`. The base size for a window icon is 16x16, but it's
    ///   recommended to account for screen scaling and pick a multiple of that, i.e. 32x32.
    ///
    /// - **X11:** Has no universal guidelines for icon sizes, so you're at the whims of the WM. That
    ///   said, it's usually in the same ballpark as on Windows.
    SetWindowIcon(Option<Icon>),
    /// Toggles whether window has decorations
    /// ## Platform-specific
    /// - **X11:** Not implemented.
    /// - **Web:** Unsupported.
    ToggleDecorations,
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
            Self::Close => Action::Close,
            Self::Drag => Action::Drag,
            Self::Resize { width, height } => Action::Resize { width, height },
            Self::Maximize(bool) => Action::Maximize(bool),
            Self::Minimize(bool) => Action::Minimize(bool),
            Self::Move { x, y } => Action::Move { x, y },
            Self::SetMode(mode) => Action::SetMode(mode),
            Self::ToggleMaximize => Action::ToggleMaximize,
            Self::SetWindowIcon(icon) => Action::SetWindowIcon(icon),
            Self::ToggleDecorations => Action::ToggleDecorations,
            Self::FetchMode(o) => Action::FetchMode(Box::new(move |s| f(o(s)))),
        }
    }
}

impl<T> fmt::Debug for Action<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Close => write!(f, "Action::Close"),
            Self::Drag => write!(f, "Action::Drag"),
            Self::Resize { width, height } => write!(
                f,
                "Action::Resize {{ widget: {}, height: {} }}",
                width, height
            ),
            Self::Maximize(value) => write!(f, "Action::Maximize({})", value),
            Self::Minimize(value) => write!(f, "Action::Minimize({}", value),
            Self::Move { x, y } => {
                write!(f, "Action::Move {{ x: {}, y: {} }}", x, y)
            }
            Self::SetMode(mode) => write!(f, "Action::SetMode({:?})", mode),
            Self::ToggleMaximize => write!(f, "Action::ToggleMaximize"),
            Self::SetWindowIcon(_icon) => {
                write!(f, "Action::SetWindowIcon(icon)")
            }
            Self::ToggleDecorations => write!(f, "Action::ToggleDecorations"),
            Self::FetchMode(_) => write!(f, "Action::FetchMode"),
        }
    }
}
