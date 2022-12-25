//! Interact with the window of your application.
use crate::command::Command;
use iced_native::window::Action;

pub use iced_native::window::{Event, Mode};

/// Closes the current window and exits the application.
pub fn close<Message>() -> Command<Message> {
    Command::Window(Action::Close)
}

/// Begins dragging the window while the left mouse button is held.
pub fn drag<Message>() -> Command<Message> {
    Command::Window(Action::Drag)
}

/// Resizes the window to the given logical dimensions.
pub fn resize<Message>(width: u32, height: u32) -> Command<Message> {
    Command::Window(Action::Resize { width, height })
}

/// Sets the window to maximized or back.
pub fn maximize<Message>(value: bool) -> Command<Message> {
    Command::Window(Action::Maximize(value))
}

/// Set the window to minimized or back.
pub fn minimize<Message>(value: bool) -> Command<Message> {
    Command::Window(Action::Minimize(value))
}

/// Moves a window to the given logical coordinates.
pub fn move_to<Message>(x: i32, y: i32) -> Command<Message> {
    Command::Window(Action::Move { x, y })
}

/// Sets the [`Mode`] of the window.
pub fn set_mode<Message>(mode: Mode) -> Command<Message> {
    Command::Window(Action::SetMode(mode))
}

/// Sets the window to maximized or back.
pub fn toggle_maximize<Message>() -> Command<Message> {
    Command::Window(Action::ToggleMaximize)
}

/// Fetches the current [`Mode`] of the window.
pub fn fetch_mode<Message>(
    f: impl FnOnce(Mode) -> Message + 'static,
) -> Command<Message> {
    Command::Window(Action::FetchMode(Box::new(f)))
}
