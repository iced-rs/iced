//! Interact with the window of your application.
use crate::command::{self, Command};
use iced_native::window;

pub use window::{Event, Id, Mode, RedrawRequest, frames, UserAttention};

/// Closes the window.
pub fn close<Message>(id: window::Id) -> Command<Message> {
    Command::single(command::Action::Window(id, window::Action::Close))
}

/// Begins dragging the window while the left mouse button is held.
pub fn drag<Message>(id: window::Id) -> Command<Message> {
    Command::single(command::Action::Window(id, window::Action::Drag))
}

/// Spawns a new window.
pub fn spawn<Message>(
    id: window::Id,
    settings: window::Settings,
) -> Command<Message> {
    Command::single(command::Action::Window(
        id,
        window::Action::Spawn { settings },
    ))
}

/// Resizes the window to the given logical dimensions.
pub fn resize<Message>(
    id: window::Id,
    width: u32,
    height: u32,
) -> Command<Message> {
    Command::single(command::Action::Window(
        id,
        window::Action::Resize { width, height },
    ))
}

/// Maximizes the window.
pub fn maximize<Message>(id: window::Id, value: bool) -> Command<Message> {
    Command::single(command::Action::Window(
        id,
        window::Action::Maximize(value),
    ))
}

/// Minimes the window.
pub fn minimize<Message>(id: window::Id, value: bool) -> Command<Message> {
    Command::single(command::Action::Window(
        id,
        window::Action::Minimize(value),
    ))
}

/// Moves a window to the given logical coordinates.
pub fn move_to<Message>(id: window::Id, x: i32, y: i32) -> Command<Message> {
    Command::single(command::Action::Window(id, window::Action::Move { x, y }))
}

/// Changes the [`Mode`] of the window.
pub fn change_mode<Message>(id: window::Id, mode: Mode) -> Command<Message> {
    Command::single(command::Action::Window(id, window::Action::ChangeMode(mode)))
}

/// Fetches the current [`Mode`] of the window.
pub fn fetch_mode<Message>(
    id: window::Id,
    f: impl FnOnce(Mode) -> Message + 'static,
) -> Command<Message> {
    Command::single(command::Action::Window(
        id,
        window::Action::FetchMode(Box::new(f)),
    ))
}

/// Toggles the window to maximized or back.
pub fn toggle_maximize<Message>(id: window::Id) -> Command<Message> {
    Command::single(command::Action::Window(id, window::Action::ToggleMaximize))
}

/// Toggles the window decorations.
pub fn toggle_decorations<Message>(id: window::Id) -> Command<Message> {
    Command::single(command::Action::Window(id, window::Action::ToggleDecorations))
}

/// Request user attention to the window, this has no effect if the application
/// is already focused. How requesting for user attention manifests is platform dependent,
/// see [`UserAttention`] for details.
///
/// Providing `None` will unset the request for user attention. Unsetting the request for
/// user attention might not be done automatically by the WM when the window receives input.
pub fn request_user_attention<Message>(
    id: window::Id,
    user_attention: Option<UserAttention>,
) -> Command<Message> {
    Command::single(command::Action::Window(
        id,
        window::Action::RequestUserAttention(user_attention),
    ))
}

/// Brings the window to the front and sets input focus. Has no effect if the window is
/// already in focus, minimized, or not visible.
///
/// This [`Command`] steals input focus from other applications. Do not use this method unless
/// you are certain that's what the user wants. Focus stealing can cause an extremely disruptive
/// user experience.
pub fn gain_focus<Message>(id: window::Id) -> Command<Message> {
    Command::single(command::Action::Window(id, window::Action::GainFocus))
}
