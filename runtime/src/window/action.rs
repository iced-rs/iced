use crate::core::window::{Icon, Level, Mode, UserAttention};
use crate::core::Size;
use crate::futures::MaybeSend;
use crate::window::Screenshot;

use std::fmt;

/// An operation to be performed on some window.
pub enum Action<T> {
    /// Close the current window and exits the application.
    Close,
    /// Move the window with the left mouse button until the button is
    /// released.
    ///
    /// There’s no guarantee that this will work unless the left mouse
    /// button was pressed immediately before this function is called.
    Drag,
    /// Resize the window.
    Resize(Size<u32>),
    /// Fetch the current size of the window.
    FetchSize(Box<dyn FnOnce(Size<u32>) -> T + 'static>),
    /// Set the window to maximized or back
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
    /// Change the [`Mode`] of the window.
    ChangeMode(Mode),
    /// Fetch the current [`Mode`] of the window.
    FetchMode(Box<dyn FnOnce(Mode) -> T + 'static>),
    /// Toggle the window to maximized or back
    ToggleMaximize,
    /// Toggle whether window has decorations.
    ///
    /// ## Platform-specific
    /// - **X11:** Not implemented.
    /// - **Web:** Unsupported.
    ToggleDecorations,
    /// Request user attention to the window, this has no effect if the application
    /// is already focused. How requesting for user attention manifests is platform dependent,
    /// see [`UserAttention`] for details.
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
    /// Bring the window to the front and sets input focus. Has no effect if the window is
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
    /// Change the window [`Level`].
    ChangeLevel(Level),
    /// Fetch an identifier unique to the window.
    FetchId(Box<dyn FnOnce(u64) -> T + 'static>),
    /// Change the window [`Icon`].
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
    ChangeIcon(Icon),
    /// Screenshot the viewport of the window.
    Screenshot(Box<dyn FnOnce(Screenshot) -> T + 'static>),
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
            Self::Resize(size) => Action::Resize(size),
            Self::FetchSize(o) => Action::FetchSize(Box::new(move |s| f(o(s)))),
            Self::Maximize(maximized) => Action::Maximize(maximized),
            Self::Minimize(minimized) => Action::Minimize(minimized),
            Self::Move { x, y } => Action::Move { x, y },
            Self::ChangeMode(mode) => Action::ChangeMode(mode),
            Self::FetchMode(o) => Action::FetchMode(Box::new(move |s| f(o(s)))),
            Self::ToggleMaximize => Action::ToggleMaximize,
            Self::ToggleDecorations => Action::ToggleDecorations,
            Self::RequestUserAttention(attention_type) => {
                Action::RequestUserAttention(attention_type)
            }
            Self::GainFocus => Action::GainFocus,
            Self::ChangeLevel(level) => Action::ChangeLevel(level),
            Self::FetchId(o) => Action::FetchId(Box::new(move |s| f(o(s)))),
            Self::ChangeIcon(icon) => Action::ChangeIcon(icon),
            Self::Screenshot(tag) => {
                Action::Screenshot(Box::new(move |screenshot| {
                    f(tag(screenshot))
                }))
            }
        }
    }
}

impl<T> fmt::Debug for Action<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Close => write!(f, "Action::Close"),
            Self::Drag => write!(f, "Action::Drag"),
            Self::Resize(size) => write!(f, "Action::Resize({size:?})"),
            Self::FetchSize(_) => write!(f, "Action::FetchSize"),
            Self::Maximize(maximized) => {
                write!(f, "Action::Maximize({maximized})")
            }
            Self::Minimize(minimized) => {
                write!(f, "Action::Minimize({minimized}")
            }
            Self::Move { x, y } => {
                write!(f, "Action::Move {{ x: {x}, y: {y} }}")
            }
            Self::ChangeMode(mode) => write!(f, "Action::SetMode({mode:?})"),
            Self::FetchMode(_) => write!(f, "Action::FetchMode"),
            Self::ToggleMaximize => write!(f, "Action::ToggleMaximize"),
            Self::ToggleDecorations => write!(f, "Action::ToggleDecorations"),
            Self::RequestUserAttention(_) => {
                write!(f, "Action::RequestUserAttention")
            }
            Self::GainFocus => write!(f, "Action::GainFocus"),
            Self::ChangeLevel(level) => {
                write!(f, "Action::ChangeLevel({level:?})")
            }
            Self::FetchId(_) => write!(f, "Action::FetchId"),
            Self::ChangeIcon(_icon) => {
                write!(f, "Action::ChangeIcon(icon)")
            }
            Self::Screenshot(_) => write!(f, "Action::Screenshot"),
        }
    }
}
