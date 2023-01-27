use crate::window::{Mode, UserAttention};

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
    /// Fetch the current [`Mode`] of the window.
    FetchMode(Box<dyn FnOnce(Mode) -> T + 'static>),
    /// Sets the window to maximized or back
    ToggleMaximize,
    /// Toggles whether window has decorations
    /// ## Platform-specific
    /// - **X11:** Not implemented.
    /// - **Web:** Unsupported.
    ToggleDecorations,
    /// Requests user attention to the window, this has no effect if the application
    /// is already focused. How requesting for user attention manifests is platform dependent,
    /// see [`UserAttentionType`] for details.
    ///
    /// Providing `None` will unset the request for user attention. Unsetting the request for
    /// user attention might not be done automatically by the WM when the window receives input.
    ///
    /// ## Platform-specific
    ///
    /// - **iOS / Android / Web:** Unsupported.
    /// - **macOS:** `None` has no effect.
    /// - **X11:** Requests for user attention must be manually cleared.
    /// - **Wayland:** Requires `xdg_activation_v1` protocol, `None` has no effect.
    RequestUserAttention(Option<UserAttention>),
    /// Brings the window to the front and sets input focus. Has no effect if the window is
    /// already in focus, minimized, or not visible.
    ///
    /// This method steals input focus from other applications. Do not use this method unless
    /// you are certain that's what the user wants. Focus stealing can cause an extremely disruptive
    /// user experience.
    ///
    /// ## Platform-specific
    ///
    /// - **Web / Wayland:** Unsupported.
    GainFocus,
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
            Self::FetchMode(o) => Action::FetchMode(Box::new(move |s| f(o(s)))),
            Self::ToggleMaximize => Action::ToggleMaximize,
            Self::ToggleDecorations => Action::ToggleDecorations,
            Self::RequestUserAttention(attention_type) => {
                Action::RequestUserAttention(attention_type)
            }
            Self::GainFocus => Action::GainFocus,
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
                "Action::Resize {{ widget: {width}, height: {height} }}"
            ),
            Self::Maximize(value) => write!(f, "Action::Maximize({value})"),
            Self::Minimize(value) => write!(f, "Action::Minimize({value}"),
            Self::Move { x, y } => {
                write!(f, "Action::Move {{ x: {x}, y: {y} }}")
            }
            Self::SetMode(mode) => write!(f, "Action::SetMode({mode:?})"),
            Self::FetchMode(_) => write!(f, "Action::FetchMode"),
            Self::ToggleMaximize => write!(f, "Action::ToggleMaximize"),
            Self::ToggleDecorations => write!(f, "Action::ToggleDecorations"),
            Self::RequestUserAttention(_) => {
                write!(f, "Action::RequestUserAttention")
            }
            Self::GainFocus => write!(f, "Action::GainFocus"),
        }
    }
}
