//! Interact with the window of your application.
use crate::command::{self, Command};
use iced_native::window;

pub use window::{frames, Event, Mode, RedrawRequest, UserAttention};

/// Closes the current window and exits the application.
pub fn close<Message>() -> Command<Message> {
    Command::single(command::Action::Window(window::Action::Close))
}

/// Begins dragging the window while the left mouse button is held.
pub fn drag<Message>() -> Command<Message> {
    Command::single(command::Action::Window(window::Action::Drag))
}

/// Resizes the window to the given logical dimensions.
pub fn resize<Message>(width: u32, height: u32) -> Command<Message> {
    Command::single(command::Action::Window(window::Action::Resize {
        width,
        height,
    }))
}

/// Maximizes the window.
pub fn maximize<Message>(maximized: bool) -> Command<Message> {
    Command::single(command::Action::Window(window::Action::Maximize(
        maximized,
    )))
}

/// Minimes the window.
pub fn minimize<Message>(minimized: bool) -> Command<Message> {
    Command::single(command::Action::Window(window::Action::Minimize(
        minimized,
    )))
}

/// Moves a window to the given logical coordinates.
pub fn move_to<Message>(x: i32, y: i32) -> Command<Message> {
    Command::single(command::Action::Window(window::Action::Move { x, y }))
}

/// Sets the [`Mode`] of the window.
pub fn change_mode<Message>(mode: Mode) -> Command<Message> {
    Command::single(command::Action::Window(window::Action::ChangeMode(mode)))
}

/// Fetches the current [`Mode`] of the window.
pub fn fetch_mode<Message>(
    f: impl FnOnce(Mode) -> Message + 'static,
) -> Command<Message> {
    Command::single(command::Action::Window(window::Action::FetchMode(
        Box::new(f),
    )))
}

/// Toggles the window to maximized or back.
pub fn toggle_maximize<Message>() -> Command<Message> {
    Command::single(command::Action::Window(window::Action::ToggleMaximize))
}

/// Toggles the window decorations.
pub fn toggle_decorations<Message>() -> Command<Message> {
    Command::single(command::Action::Window(window::Action::ToggleDecorations))
}

/// Request user attention to the window, this has no effect if the application
/// is already focused. How requesting for user attention manifests is platform dependent,
/// see [`UserAttention`] for details.
///
/// Providing `None` will unset the request for user attention. Unsetting the request for
/// user attention might not be done automatically by the WM when the window receives input.
pub fn request_user_attention<Message>(
    user_attention: Option<UserAttention>,
) -> Command<Message> {
    Command::single(command::Action::Window(
        window::Action::RequestUserAttention(user_attention),
    ))
}

/// Brings the window to the front and sets input focus. Has no effect if the window is
/// already in focus, minimized, or not visible.
///
/// This [`Command`] steals input focus from other applications. Do not use this method unless
/// you are certain that's what the user wants. Focus stealing can cause an extremely disruptive
/// user experience.
pub fn gain_focus<Message>() -> Command<Message> {
    Command::single(command::Action::Window(window::Action::GainFocus))
}

/// Changes whether or not the window will always be on top of other windows.
pub fn change_always_on_top<Message>(on_top: bool) -> Command<Message> {
    Command::single(command::Action::Window(window::Action::ChangeAlwaysOnTop(
        on_top,
    )))
}

/// Fetches an identifier unique to the window.
pub fn fetch_id<Message>(
    f: impl FnOnce(u64) -> Message + 'static,
) -> Command<Message> {
    Command::single(command::Action::Window(window::Action::FetchId(Box::new(
        f,
    ))))
}
