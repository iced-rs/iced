use crate::core::window::{Icon, Id, Level, Mode, Settings, UserAttention};
use crate::core::{Point, Size};
use crate::futures::MaybeSend;
use crate::window::Screenshot;

use raw_window_handle::WindowHandle;

use std::fmt;

/// An operation to be performed on some window.
pub enum Action<T> {
    /// Spawns a new window with some [`Settings`].
    Spawn(Id, Settings),
    /// Close the window and exits the application.
    Close(Id),
    /// Move the window with the left mouse button until the button is
    /// released.
    ///
    /// There’s no guarantee that this will work unless the left mouse
    /// button was pressed immediately before this function is called.
    Drag(Id),
    /// Resize the window to the given logical dimensions.
    Resize(Id, Size),
    /// Fetch the current logical dimensions of the window.
    FetchSize(Id, Box<dyn FnOnce(Size) -> T + 'static>),
    /// Fetch if the current window is maximized or not.
    ///
    /// ## Platform-specific
    /// - **iOS / Android / Web:** Unsupported.
    FetchMaximized(Id, Box<dyn FnOnce(bool) -> T + 'static>),
    /// Set the window to maximized or back
    Maximize(Id, bool),
    /// Fetch if the current window is minimized or not.
    ///
    /// ## Platform-specific
    /// - **Wayland:** Always `None`.
    /// - **iOS / Android / Web:** Unsupported.
    FetchMinimized(Id, Box<dyn FnOnce(Option<bool>) -> T + 'static>),
    /// Set the window to minimized or back
    Minimize(Id, bool),
    /// Move the window to the given logical coordinates.
    ///
    /// Unsupported on Wayland.
    Move(Id, Point),
    /// Change the [`Mode`] of the window.
    ChangeMode(Id, Mode),
    /// Fetch the current [`Mode`] of the window.
    FetchMode(Id, Box<dyn FnOnce(Mode) -> T + 'static>),
    /// Toggle the window to maximized or back
    ToggleMaximize(Id),
    /// Toggle whether window has decorations.
    ///
    /// ## Platform-specific
    /// - **X11:** Not implemented.
    /// - **Web:** Unsupported.
    ToggleDecorations(Id),
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
    RequestUserAttention(Id, Option<UserAttention>),
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
    GainFocus(Id),
    /// Change the window [`Level`].
    ChangeLevel(Id, Level),
    /// Show the system menu at cursor position.
    ///
    /// ## Platform-specific
    /// Android / iOS / macOS / Orbital / Web / X11: Unsupported.
    ShowSystemMenu(Id),
    /// Fetch the raw identifier unique to the window.
    FetchId(Id, Box<dyn FnOnce(u64) -> T + 'static>),
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
    ChangeIcon(Id, Icon),
    /// Runs the closure with the native window handle of the window with the given [`Id`].
    RunWithHandle(Id, Box<dyn FnOnce(&WindowHandle<'_>) -> T + 'static>),
    /// Screenshot the viewport of the window.
    Screenshot(Id, Box<dyn FnOnce(Screenshot) -> T + 'static>),
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
            Self::Spawn(id, settings) => Action::Spawn(id, settings),
            Self::Close(id) => Action::Close(id),
            Self::Drag(id) => Action::Drag(id),
            Self::Resize(id, size) => Action::Resize(id, size),
            Self::FetchSize(id, o) => {
                Action::FetchSize(id, Box::new(move |s| f(o(s))))
            }
            Self::FetchMaximized(id, o) => {
                Action::FetchMaximized(id, Box::new(move |s| f(o(s))))
            }
            Self::Maximize(id, maximized) => Action::Maximize(id, maximized),
            Self::FetchMinimized(id, o) => {
                Action::FetchMinimized(id, Box::new(move |s| f(o(s))))
            }
            Self::Minimize(id, minimized) => Action::Minimize(id, minimized),
            Self::Move(id, position) => Action::Move(id, position),
            Self::ChangeMode(id, mode) => Action::ChangeMode(id, mode),
            Self::FetchMode(id, o) => {
                Action::FetchMode(id, Box::new(move |s| f(o(s))))
            }
            Self::ToggleMaximize(id) => Action::ToggleMaximize(id),
            Self::ToggleDecorations(id) => Action::ToggleDecorations(id),
            Self::RequestUserAttention(id, attention_type) => {
                Action::RequestUserAttention(id, attention_type)
            }
            Self::GainFocus(id) => Action::GainFocus(id),
            Self::ChangeLevel(id, level) => Action::ChangeLevel(id, level),
            Self::ShowSystemMenu(id) => Action::ShowSystemMenu(id),
            Self::FetchId(id, o) => {
                Action::FetchId(id, Box::new(move |s| f(o(s))))
            }
            Self::ChangeIcon(id, icon) => Action::ChangeIcon(id, icon),
            Self::RunWithHandle(id, o) => {
                Action::RunWithHandle(id, Box::new(move |s| f(o(s))))
            }
            Self::Screenshot(id, tag) => Action::Screenshot(
                id,
                Box::new(move |screenshot| f(tag(screenshot))),
            ),
        }
    }
}

impl<T> fmt::Debug for Action<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Spawn(id, settings) => {
                write!(f, "Action::Spawn({id:?}, {settings:?})")
            }
            Self::Close(id) => write!(f, "Action::Close({id:?})"),
            Self::Drag(id) => write!(f, "Action::Drag({id:?})"),
            Self::Resize(id, size) => {
                write!(f, "Action::Resize({id:?}, {size:?})")
            }
            Self::FetchSize(id, _) => write!(f, "Action::FetchSize({id:?})"),
            Self::FetchMaximized(id, _) => {
                write!(f, "Action::FetchMaximized({id:?})")
            }
            Self::Maximize(id, maximized) => {
                write!(f, "Action::Maximize({id:?}, {maximized})")
            }
            Self::FetchMinimized(id, _) => {
                write!(f, "Action::FetchMinimized({id:?})")
            }
            Self::Minimize(id, minimized) => {
                write!(f, "Action::Minimize({id:?}, {minimized}")
            }
            Self::Move(id, position) => {
                write!(f, "Action::Move({id:?}, {position})")
            }
            Self::ChangeMode(id, mode) => {
                write!(f, "Action::SetMode({id:?}, {mode:?})")
            }
            Self::FetchMode(id, _) => write!(f, "Action::FetchMode({id:?})"),
            Self::ToggleMaximize(id) => {
                write!(f, "Action::ToggleMaximize({id:?})")
            }
            Self::ToggleDecorations(id) => {
                write!(f, "Action::ToggleDecorations({id:?})")
            }
            Self::RequestUserAttention(id, _) => {
                write!(f, "Action::RequestUserAttention({id:?})")
            }
            Self::GainFocus(id) => write!(f, "Action::GainFocus({id:?})"),
            Self::ChangeLevel(id, level) => {
                write!(f, "Action::ChangeLevel({id:?}, {level:?})")
            }
            Self::ShowSystemMenu(id) => {
                write!(f, "Action::ShowSystemMenu({id:?})")
            }
            Self::FetchId(id, _) => write!(f, "Action::FetchId({id:?})"),
            Self::ChangeIcon(id, _icon) => {
                write!(f, "Action::ChangeIcon({id:?})")
            }
            Self::RunWithHandle(id, _) => {
                write!(f, "Action::RunWithHandle({id:?})")
            }
            Self::Screenshot(id, _) => write!(f, "Action::Screenshot({id:?})"),
        }
    }
}
