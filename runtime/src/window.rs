//! Build window-based GUI applications.
mod action;

pub mod screenshot;

pub use crate::core::window::Id;
pub use action::Action;
pub use screenshot::Screenshot;

use crate::command::{self, Command};
use crate::core::time::Instant;
use crate::core::window::{self, Event, Icon, Level, Mode, UserAttention};
use crate::core::Size;
use crate::futures::subscription::{self, Subscription};

/// Subscribes to the frames of the window of the running application.
///
/// The resulting [`Subscription`] will produce items at a rate equal to the
/// refresh rate of the first application window. Note that this rate may be variable, as it is
/// normally managed by the graphics driver and/or the OS.
///
/// In any case, this [`Subscription`] is useful to smoothly draw application-driven
/// animations without missing any frames.
pub fn frames() -> Subscription<Instant> {
    subscription::raw_events(|event, _status| match event {
        iced_core::Event::Window(_, Event::RedrawRequested(at)) => Some(at),
        _ => None,
    })
}

/// Spawns a new window with the given `id` and `settings`.
pub fn spawn<Message>(
    id: window::Id,
    settings: window::Settings,
) -> Command<Message> {
    Command::single(command::Action::Window(id, Action::Spawn { settings }))
}

/// Closes the window with `id`.
pub fn close<Message>(id: window::Id) -> Command<Message> {
    Command::single(command::Action::Window(id, Action::Close))
}

/// Begins dragging the window while the left mouse button is held.
pub fn drag<Message>(id: window::Id) -> Command<Message> {
    Command::single(command::Action::Window(id, Action::Drag))
}

/// Resizes the window to the given logical dimensions.
pub fn resize<Message>(
    id: window::Id,
    new_size: Size<u32>,
) -> Command<Message> {
    Command::single(command::Action::Window(id, Action::Resize(new_size)))
}

/// Fetches the window's size in logical dimensions.
pub fn fetch_size<Message>(
    id: window::Id,
    f: impl FnOnce(Size<u32>) -> Message + 'static,
) -> Command<Message> {
    Command::single(command::Action::Window(id, Action::FetchSize(Box::new(f))))
}

/// Maximizes the window.
pub fn maximize<Message>(id: window::Id, maximized: bool) -> Command<Message> {
    Command::single(command::Action::Window(id, Action::Maximize(maximized)))
}

/// Minimizes the window.
pub fn minimize<Message>(id: window::Id, minimized: bool) -> Command<Message> {
    Command::single(command::Action::Window(id, Action::Minimize(minimized)))
}

/// Moves the window to the given logical coordinates.
pub fn move_to<Message>(id: window::Id, x: i32, y: i32) -> Command<Message> {
    Command::single(command::Action::Window(id, Action::Move { x, y }))
}

/// Changes the [`Mode`] of the window.
pub fn change_mode<Message>(id: window::Id, mode: Mode) -> Command<Message> {
    Command::single(command::Action::Window(id, Action::ChangeMode(mode)))
}

/// Fetches the current [`Mode`] of the window.
pub fn fetch_mode<Message>(
    id: window::Id,
    f: impl FnOnce(Mode) -> Message + 'static,
) -> Command<Message> {
    Command::single(command::Action::Window(id, Action::FetchMode(Box::new(f))))
}

/// Toggles the window to maximized or back.
pub fn toggle_maximize<Message>(id: window::Id) -> Command<Message> {
    Command::single(command::Action::Window(id, Action::ToggleMaximize))
}

/// Toggles the window decorations.
pub fn toggle_decorations<Message>(id: window::Id) -> Command<Message> {
    Command::single(command::Action::Window(id, Action::ToggleDecorations))
}

/// Request user attention to the window. This has no effect if the application
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
        Action::RequestUserAttention(user_attention),
    ))
}

/// Brings the window to the front and sets input focus. Has no effect if the window is
/// already in focus, minimized, or not visible.
///
/// This [`Command`] steals input focus from other applications. Do not use this method unless
/// you are certain that's what the user wants. Focus stealing can cause an extremely disruptive
/// user experience.
pub fn gain_focus<Message>(id: window::Id) -> Command<Message> {
    Command::single(command::Action::Window(id, Action::GainFocus))
}

/// Changes the window [`Level`].
pub fn change_level<Message>(id: window::Id, level: Level) -> Command<Message> {
    Command::single(command::Action::Window(id, Action::ChangeLevel(level)))
}

/// Fetches an identifier unique to the window, provided by the underlying windowing system. This is
/// not to be confused with [`window::Id`].
pub fn fetch_id<Message>(
    id: window::Id,
    f: impl FnOnce(u64) -> Message + 'static,
) -> Command<Message> {
    Command::single(command::Action::Window(id, Action::FetchId(Box::new(f))))
}

/// Changes the [`Icon`] of the window.
pub fn change_icon<Message>(id: window::Id, icon: Icon) -> Command<Message> {
    Command::single(command::Action::Window(id, Action::ChangeIcon(icon)))
}

/// Captures a [`Screenshot`] from the window.
pub fn screenshot<Message>(
    id: window::Id,
    f: impl FnOnce(Screenshot) -> Message + Send + 'static,
) -> Command<Message> {
    Command::single(command::Action::Window(
        id,
        Action::Screenshot(Box::new(f)),
    ))
}
