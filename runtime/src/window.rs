//! Build window-based GUI applications.
mod action;

pub mod screenshot;

pub use action::Action;
pub use screenshot::Screenshot;

use crate::command::{self, Command};
use crate::core::time::Instant;
use crate::core::window::{
    Event, Icon, Id, Level, Mode, Settings, UserAttention,
};
use crate::core::{Point, Size};
use crate::futures::event;
use crate::futures::Subscription;

pub use raw_window_handle;

use raw_window_handle::WindowHandle;

/// Subscribes to the frames of the window of the running application.
///
/// The resulting [`Subscription`] will produce items at a rate equal to the
/// refresh rate of the first application window. Note that this rate may be variable, as it is
/// normally managed by the graphics driver and/or the OS.
///
/// In any case, this [`Subscription`] is useful to smoothly draw application-driven
/// animations without missing any frames.
pub fn frames() -> Subscription<Instant> {
    event::listen_raw(|event, _status| match event {
        crate::core::Event::Window(_, Event::RedrawRequested(at)) => Some(at),
        _ => None,
    })
}

/// Spawns a new window with the given `settings`.
///
/// Returns the new window [`Id`] alongside the [`Command`].
pub fn spawn<Message>(settings: Settings) -> (Id, Command<Message>) {
    let id = Id::unique();

    (
        id,
        Command::single(command::Action::Window(Action::Spawn(id, settings))),
    )
}

/// Closes the window with `id`.
pub fn close<Message>(id: Id) -> Command<Message> {
    Command::single(command::Action::Window(Action::Close(id)))
}

/// Begins dragging the window while the left mouse button is held.
pub fn drag<Message>(id: Id) -> Command<Message> {
    Command::single(command::Action::Window(Action::Drag(id)))
}

/// Resizes the window to the given logical dimensions.
pub fn resize<Message>(id: Id, new_size: Size) -> Command<Message> {
    Command::single(command::Action::Window(Action::Resize(id, new_size)))
}

/// Fetches the window's size in logical dimensions.
pub fn fetch_size<Message>(
    id: Id,
    f: impl FnOnce(Size) -> Message + 'static,
) -> Command<Message> {
    Command::single(command::Action::Window(Action::FetchSize(id, Box::new(f))))
}

/// Fetches if the window is maximized.
pub fn fetch_maximized<Message>(
    id: Id,
    f: impl FnOnce(bool) -> Message + 'static,
) -> Command<Message> {
    Command::single(command::Action::Window(Action::FetchMaximized(
        id,
        Box::new(f),
    )))
}

/// Maximizes the window.
pub fn maximize<Message>(id: Id, maximized: bool) -> Command<Message> {
    Command::single(command::Action::Window(Action::Maximize(id, maximized)))
}

/// Fetches if the window is minimized.
pub fn fetch_minimized<Message>(
    id: Id,
    f: impl FnOnce(Option<bool>) -> Message + 'static,
) -> Command<Message> {
    Command::single(command::Action::Window(Action::FetchMinimized(
        id,
        Box::new(f),
    )))
}

/// Minimizes the window.
pub fn minimize<Message>(id: Id, minimized: bool) -> Command<Message> {
    Command::single(command::Action::Window(Action::Minimize(id, minimized)))
}

/// Moves the window to the given logical coordinates.
pub fn move_to<Message>(id: Id, position: Point) -> Command<Message> {
    Command::single(command::Action::Window(Action::Move(id, position)))
}

/// Changes the [`Mode`] of the window.
pub fn change_mode<Message>(id: Id, mode: Mode) -> Command<Message> {
    Command::single(command::Action::Window(Action::ChangeMode(id, mode)))
}

/// Fetches the current [`Mode`] of the window.
pub fn fetch_mode<Message>(
    id: Id,
    f: impl FnOnce(Mode) -> Message + 'static,
) -> Command<Message> {
    Command::single(command::Action::Window(Action::FetchMode(id, Box::new(f))))
}

/// Toggles the window to maximized or back.
pub fn toggle_maximize<Message>(id: Id) -> Command<Message> {
    Command::single(command::Action::Window(Action::ToggleMaximize(id)))
}

/// Toggles the window decorations.
pub fn toggle_decorations<Message>(id: Id) -> Command<Message> {
    Command::single(command::Action::Window(Action::ToggleDecorations(id)))
}

/// Request user attention to the window. This has no effect if the application
/// is already focused. How requesting for user attention manifests is platform dependent,
/// see [`UserAttention`] for details.
///
/// Providing `None` will unset the request for user attention. Unsetting the request for
/// user attention might not be done automatically by the WM when the window receives input.
pub fn request_user_attention<Message>(
    id: Id,
    user_attention: Option<UserAttention>,
) -> Command<Message> {
    Command::single(command::Action::Window(Action::RequestUserAttention(
        id,
        user_attention,
    )))
}

/// Brings the window to the front and sets input focus. Has no effect if the window is
/// already in focus, minimized, or not visible.
///
/// This [`Command`] steals input focus from other applications. Do not use this method unless
/// you are certain that's what the user wants. Focus stealing can cause an extremely disruptive
/// user experience.
pub fn gain_focus<Message>(id: Id) -> Command<Message> {
    Command::single(command::Action::Window(Action::GainFocus(id)))
}

/// Changes the window [`Level`].
pub fn change_level<Message>(id: Id, level: Level) -> Command<Message> {
    Command::single(command::Action::Window(Action::ChangeLevel(id, level)))
}

/// Show the [system menu] at cursor position.
///
/// [system menu]: https://en.wikipedia.org/wiki/Common_menus_in_Microsoft_Windows#System_menu
pub fn show_system_menu<Message>(id: Id) -> Command<Message> {
    Command::single(command::Action::Window(Action::ShowSystemMenu(id)))
}

/// Fetches an identifier unique to the window, provided by the underlying windowing system. This is
/// not to be confused with [`Id`].
pub fn fetch_id<Message>(
    id: Id,
    f: impl FnOnce(u64) -> Message + 'static,
) -> Command<Message> {
    Command::single(command::Action::Window(Action::FetchId(id, Box::new(f))))
}

/// Changes the [`Icon`] of the window.
pub fn change_icon<Message>(id: Id, icon: Icon) -> Command<Message> {
    Command::single(command::Action::Window(Action::ChangeIcon(id, icon)))
}

/// Runs the given callback with the native window handle for the window with the given id.
///
/// Note that if the window closes before this call is processed the callback will not be run.
pub fn run_with_handle<Message>(
    id: Id,
    f: impl FnOnce(&WindowHandle<'_>) -> Message + 'static,
) -> Command<Message> {
    Command::single(command::Action::Window(Action::RunWithHandle(
        id,
        Box::new(f),
    )))
}

/// Captures a [`Screenshot`] from the window.
pub fn screenshot<Message>(
    id: Id,
    f: impl FnOnce(Screenshot) -> Message + Send + 'static,
) -> Command<Message> {
    Command::single(command::Action::Window(Action::Screenshot(
        id,
        Box::new(f),
    )))
}
