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
pub fn maximize<Message>(value: bool) -> Command<Message> {
    Command::single(command::Action::Window(window::Action::Maximize(value)))
}

/// Minimes the window.
pub fn minimize<Message>(value: bool) -> Command<Message> {
    Command::single(command::Action::Window(window::Action::Minimize(value)))
}

/// Moves a window to the given logical coordinates.
pub fn move_to<Message>(x: i32, y: i32) -> Command<Message> {
    Command::single(command::Action::Window(window::Action::Move { x, y }))
}

/// Sets the [`Mode`] of the window.
pub fn change_mode<Message>(mode: Mode) -> Command<Message> {
    Command::single(command::Action::Window(window::Action::ChangeMode(mode)))
}

/// Toggles the window to maximized or back.
pub fn toggle_maximize<Message>() -> Command<Message> {
    Command::single(command::Action::Window(window::Action::ToggleMaximize))
}

/// Fetches the current [`Mode`] of the window.
pub fn fetch_mode<Message>(
    f: impl FnOnce(Mode) -> Message + 'static,
) -> Command<Message> {
    Command::single(command::Action::Window(window::Action::FetchMode(
        Box::new(f),
    )))
}
