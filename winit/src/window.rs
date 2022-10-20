//! Interact with the window of your application.
use crate::command::{self, Command};
use iced_native::window;

pub use window::{Id, Event, Mode, UserAttention};

/// Closes the current window and exits the application.
pub fn close<Message>() -> Command<Message> {
    Command::single(command::Action::Window(window::Action::Close))
}

/// Begins dragging the window while the left mouse button is held.
pub fn drag<Message>() -> Command<Message> {
    Command::single(command::Action::Window(window::Action::Drag))
}

/// TODO(derezzedex)
pub fn spawn<Message>(
    id: window::Id,
    settings: window::Settings,
) -> Command<Message> {
    Command::single(command::Action::Window(
        id,
        window::Action::Spawn { settings },
    ))
}

/// TODO(derezzedex)
pub fn close<Message>(id: window::Id) -> Command<Message> {
    Command::single(command::Action::Window(id, window::Action::Close))
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

/// Sets the window to maximized or back.
pub fn maximize<Message>(value: bool) -> Command<Message> {
    Command::single(command::Action::Window(window::Action::Maximize(value)))
}

/// Set the window to minimized or back.
pub fn minimize<Message>(value: bool) -> Command<Message> {
    Command::single(command::Action::Window(window::Action::Minimize(value)))
}

/// Moves a window to the given logical coordinates.
pub fn move_to<Message>(id: window::Id, x: i32, y: i32) -> Command<Message> {
    Command::single(command::Action::Window(id, window::Action::Move { x, y }))
}

/// Sets the [`Mode`] of the window.
pub fn set_mode<Message>(id: window::Id, mode: Mode) -> Command<Message> {
    Command::single(command::Action::Window(id, window::Action::SetMode(mode)))
}

/// Sets the window to maximized or back.
pub fn toggle_maximize<Message>() -> Command<Message> {
    Command::single(command::Action::Window(window::Action::ToggleMaximize))
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
