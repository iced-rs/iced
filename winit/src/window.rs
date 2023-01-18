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

/// Sets the window to maximized or back.
pub fn maximize<Message>(id: window::Id, value: bool) -> Command<Message> {
    Command::single(command::Action::Window(
        id,
        window::Action::Maximize(value),
    ))
}

/// Set the window to minimized or back.
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

/// Sets the [`Mode`] of the window.
pub fn set_mode<Message>(id: window::Id, mode: Mode) -> Command<Message> {
    Command::single(command::Action::Window(id, window::Action::SetMode(mode)))
}

/// Sets the window to maximized or back.
pub fn toggle_maximize<Message>(id: window::Id) -> Command<Message> {
    Command::single(command::Action::Window(id, window::Action::ToggleMaximize))
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
