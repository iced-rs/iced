//! Build window-based GUI applications.
pub mod screenshot;

pub use screenshot::Screenshot;

use crate::core::time::Instant;
use crate::core::window::{
    Event, Icon, Id, Level, Mode, Settings, UserAttention,
};
use crate::core::{Point, Size};
use crate::futures::event;
use crate::futures::futures::channel::oneshot;
use crate::futures::Subscription;
use crate::task::{self, Task};

pub use raw_window_handle;

use raw_window_handle::WindowHandle;

/// An operation to be performed on some window.
#[allow(missing_debug_implementations)]
pub enum Action {
    /// Opens a new window with some [`Settings`].
    Open(Id, Settings, oneshot::Sender<Id>),

    /// Close the window and exits the application.
    Close(Id),

    /// Gets the [`Id`] of the oldest window.
    GetOldest(oneshot::Sender<Option<Id>>),

    /// Gets the [`Id`] of the latest window.
    GetLatest(oneshot::Sender<Option<Id>>),

    /// Move the window with the left mouse button until the button is
    /// released.
    ///
    /// There’s no guarantee that this will work unless the left mouse
    /// button was pressed immediately before this function is called.
    Drag(Id),

    /// Resize the window to the given logical dimensions.
    Resize(Id, Size),

    /// Get the current logical dimensions of the window.
    GetSize(Id, oneshot::Sender<Size>),

    /// Get if the current window is maximized or not.
    GetMaximized(Id, oneshot::Sender<bool>),

    /// Set the window to maximized or back
    Maximize(Id, bool),

    /// Get if the current window is minimized or not.
    ///
    /// ## Platform-specific
    /// - **Wayland:** Always `None`.
    GetMinimized(Id, oneshot::Sender<Option<bool>>),

    /// Set the window to minimized or back
    Minimize(Id, bool),

    /// Get the current logical coordinates of the window.
    GetPosition(Id, oneshot::Sender<Option<Point>>),

    /// Move the window to the given logical coordinates.
    ///
    /// Unsupported on Wayland.
    Move(Id, Point),

    /// Change the [`Mode`] of the window.
    ChangeMode(Id, Mode),

    /// Get the current [`Mode`] of the window.
    GetMode(Id, oneshot::Sender<Mode>),

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

    /// Get the raw identifier unique to the window.
    GetRawId(Id, oneshot::Sender<u64>),

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
    RunWithHandle(Id, Box<dyn FnOnce(WindowHandle<'_>) + Send>),

    /// Screenshot the viewport of the window.
    Screenshot(Id, oneshot::Sender<Screenshot>),
}

/// Subscribes to the frames of the window of the running application.
///
/// The resulting [`Subscription`] will produce items at a rate equal to the
/// refresh rate of the first application window. Note that this rate may be variable, as it is
/// normally managed by the graphics driver and/or the OS.
///
/// In any case, this [`Subscription`] is useful to smoothly draw application-driven
/// animations without missing any frames.
pub fn frames() -> Subscription<Instant> {
    event::listen_raw(|event, _status, _window| match event {
        crate::core::Event::Window(Event::RedrawRequested(at)) => Some(at),
        _ => None,
    })
}

/// Subscribes to all window events of the running application.
pub fn events() -> Subscription<(Id, Event)> {
    event::listen_with(|event, _status, id| {
        if let crate::core::Event::Window(event) = event {
            Some((id, event))
        } else {
            None
        }
    })
}

/// Subscribes to all [`Event::Closed`] occurrences in the running application.
pub fn open_events() -> Subscription<Id> {
    event::listen_with(|event, _status, id| {
        if let crate::core::Event::Window(Event::Opened { .. }) = event {
            Some(id)
        } else {
            None
        }
    })
}

/// Subscribes to all [`Event::Closed`] occurrences in the running application.
pub fn close_events() -> Subscription<Id> {
    event::listen_with(|event, _status, id| {
        if let crate::core::Event::Window(Event::Closed) = event {
            Some(id)
        } else {
            None
        }
    })
}

/// Subscribes to all [`Event::Resized`] occurrences in the running application.
pub fn resize_events() -> Subscription<(Id, Size)> {
    event::listen_with(|event, _status, id| {
        if let crate::core::Event::Window(Event::Resized(size)) = event {
            Some((id, size))
        } else {
            None
        }
    })
}

/// Subscribes to all [`Event::CloseRequested`] occurences in the running application.
pub fn close_requests() -> Subscription<Id> {
    event::listen_with(|event, _status, id| {
        if let crate::core::Event::Window(Event::CloseRequested) = event {
            Some(id)
        } else {
            None
        }
    })
}

/// Opens a new window with the given [`Settings`]; producing the [`Id`]
/// of the new window on completion.
pub fn open(settings: Settings) -> Task<Id> {
    let id = Id::unique();

    task::oneshot(|channel| {
        crate::Action::Window(Action::Open(id, settings, channel))
    })
}

/// Closes the window with `id`.
pub fn close<T>(id: Id) -> Task<T> {
    task::effect(crate::Action::Window(Action::Close(id)))
}

/// Gets the window [`Id`] of the oldest window.
pub fn get_oldest() -> Task<Option<Id>> {
    task::oneshot(|channel| crate::Action::Window(Action::GetOldest(channel)))
}

/// Gets the window [`Id`] of the latest window.
pub fn get_latest() -> Task<Option<Id>> {
    task::oneshot(|channel| crate::Action::Window(Action::GetLatest(channel)))
}

/// Begins dragging the window while the left mouse button is held.
pub fn drag<T>(id: Id) -> Task<T> {
    task::effect(crate::Action::Window(Action::Drag(id)))
}

/// Resizes the window to the given logical dimensions.
pub fn resize<T>(id: Id, new_size: Size) -> Task<T> {
    task::effect(crate::Action::Window(Action::Resize(id, new_size)))
}

/// Get the window's size in logical dimensions.
pub fn get_size(id: Id) -> Task<Size> {
    task::oneshot(move |channel| {
        crate::Action::Window(Action::GetSize(id, channel))
    })
}

/// Gets the maximized state of the window with the given [`Id`].
pub fn get_maximized(id: Id) -> Task<bool> {
    task::oneshot(move |channel| {
        crate::Action::Window(Action::GetMaximized(id, channel))
    })
}

/// Maximizes the window.
pub fn maximize<T>(id: Id, maximized: bool) -> Task<T> {
    task::effect(crate::Action::Window(Action::Maximize(id, maximized)))
}

/// Gets the minimized state of the window with the given [`Id`].
pub fn get_minimized(id: Id) -> Task<Option<bool>> {
    task::oneshot(move |channel| {
        crate::Action::Window(Action::GetMinimized(id, channel))
    })
}

/// Minimizes the window.
pub fn minimize<T>(id: Id, minimized: bool) -> Task<T> {
    task::effect(crate::Action::Window(Action::Minimize(id, minimized)))
}

/// Gets the position in logical coordinates of the window with the given [`Id`].
pub fn get_position(id: Id) -> Task<Option<Point>> {
    task::oneshot(move |channel| {
        crate::Action::Window(Action::GetPosition(id, channel))
    })
}

/// Moves the window to the given logical coordinates.
pub fn move_to<T>(id: Id, position: Point) -> Task<T> {
    task::effect(crate::Action::Window(Action::Move(id, position)))
}

/// Changes the [`Mode`] of the window.
pub fn change_mode<T>(id: Id, mode: Mode) -> Task<T> {
    task::effect(crate::Action::Window(Action::ChangeMode(id, mode)))
}

/// Gets the current [`Mode`] of the window.
pub fn get_mode(id: Id) -> Task<Mode> {
    task::oneshot(move |channel| {
        crate::Action::Window(Action::GetMode(id, channel))
    })
}

/// Toggles the window to maximized or back.
pub fn toggle_maximize<T>(id: Id) -> Task<T> {
    task::effect(crate::Action::Window(Action::ToggleMaximize(id)))
}

/// Toggles the window decorations.
pub fn toggle_decorations<T>(id: Id) -> Task<T> {
    task::effect(crate::Action::Window(Action::ToggleDecorations(id)))
}

/// Request user attention to the window. This has no effect if the application
/// is already focused. How requesting for user attention manifests is platform dependent,
/// see [`UserAttention`] for details.
///
/// Providing `None` will unset the request for user attention. Unsetting the request for
/// user attention might not be done automatically by the WM when the window receives input.
pub fn request_user_attention<T>(
    id: Id,
    user_attention: Option<UserAttention>,
) -> Task<T> {
    task::effect(crate::Action::Window(Action::RequestUserAttention(
        id,
        user_attention,
    )))
}

/// Brings the window to the front and sets input focus. Has no effect if the window is
/// already in focus, minimized, or not visible.
///
/// This [`Task`] steals input focus from other applications. Do not use this method unless
/// you are certain that's what the user wants. Focus stealing can cause an extremely disruptive
/// user experience.
pub fn gain_focus<T>(id: Id) -> Task<T> {
    task::effect(crate::Action::Window(Action::GainFocus(id)))
}

/// Changes the window [`Level`].
pub fn change_level<T>(id: Id, level: Level) -> Task<T> {
    task::effect(crate::Action::Window(Action::ChangeLevel(id, level)))
}

/// Show the [system menu] at cursor position.
///
/// [system menu]: https://en.wikipedia.org/wiki/Common_menus_in_Microsoft_Windows#System_menu
pub fn show_system_menu<T>(id: Id) -> Task<T> {
    task::effect(crate::Action::Window(Action::ShowSystemMenu(id)))
}

/// Gets an identifier unique to the window, provided by the underlying windowing system. This is
/// not to be confused with [`Id`].
pub fn get_raw_id<Message>(id: Id) -> Task<u64> {
    task::oneshot(|channel| {
        crate::Action::Window(Action::GetRawId(id, channel))
    })
}

/// Changes the [`Icon`] of the window.
pub fn change_icon<T>(id: Id, icon: Icon) -> Task<T> {
    task::effect(crate::Action::Window(Action::ChangeIcon(id, icon)))
}

/// Runs the given callback with the native window handle for the window with the given id.
///
/// Note that if the window closes before this call is processed the callback will not be run.
pub fn run_with_handle<T>(
    id: Id,
    f: impl FnOnce(WindowHandle<'_>) -> T + Send + 'static,
) -> Task<T>
where
    T: Send + 'static,
{
    task::oneshot(move |channel| {
        crate::Action::Window(Action::RunWithHandle(
            id,
            Box::new(move |handle| {
                let _ = channel.send(f(handle));
            }),
        ))
    })
}

/// Captures a [`Screenshot`] from the window.
pub fn screenshot(id: Id) -> Task<Screenshot> {
    task::oneshot(move |channel| {
        crate::Action::Window(Action::Screenshot(id, channel))
    })
}
