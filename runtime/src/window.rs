//! Build window-based GUI applications.
mod action;

pub use crate::core::window::Id;
pub use action::Action;

use crate::command::{self, Command};
use crate::core::time::Instant;
use crate::core::window::{Event, Icon, Mode, UserAttention};
use crate::futures::subscription::{self, Subscription};

/// Subscribes to the frames of the window of the running application.
///
/// The resulting [`Subscription`] will produce items at a rate equal to the
/// refresh rate of the window. Note that this rate may be variable, as it is
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

/// Closes the current window and exits the application.
pub fn close<Message>() -> Command<Message> {
    Command::single(command::Action::Window(Action::Close))
}

/// Begins dragging the window while the left mouse button is held.
pub fn drag<Message>() -> Command<Message> {
    Command::single(command::Action::Window(Action::Drag))
}

/// Resizes the window to the given logical dimensions.
pub fn resize<Message>(width: u32, height: u32) -> Command<Message> {
    Command::single(command::Action::Window(Action::Resize { width, height }))
}

/// Maximizes the window.
pub fn maximize<Message>(maximized: bool) -> Command<Message> {
    Command::single(command::Action::Window(Action::Maximize(maximized)))
}

/// Minimes the window.
pub fn minimize<Message>(minimized: bool) -> Command<Message> {
    Command::single(command::Action::Window(Action::Minimize(minimized)))
}

/// Moves a window to the given logical coordinates.
pub fn move_to<Message>(x: i32, y: i32) -> Command<Message> {
    Command::single(command::Action::Window(Action::Move { x, y }))
}

/// Sets the [`Mode`] of the window.
pub fn change_mode<Message>(mode: Mode) -> Command<Message> {
    Command::single(command::Action::Window(Action::ChangeMode(mode)))
}

/// Fetches the current [`Mode`] of the window.
pub fn fetch_mode<Message>(
    f: impl FnOnce(Mode) -> Message + 'static,
) -> Command<Message> {
    Command::single(command::Action::Window(Action::FetchMode(Box::new(f))))
}

/// Toggles the window to maximized or back.
pub fn toggle_maximize<Message>() -> Command<Message> {
    Command::single(command::Action::Window(Action::ToggleMaximize))
}

/// Toggles the window decorations.
pub fn toggle_decorations<Message>() -> Command<Message> {
    Command::single(command::Action::Window(Action::ToggleDecorations))
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
    Command::single(command::Action::Window(Action::RequestUserAttention(
        user_attention,
    )))
}

/// Brings the window to the front and sets input focus. Has no effect if the window is
/// already in focus, minimized, or not visible.
///
/// This [`Command`] steals input focus from other applications. Do not use this method unless
/// you are certain that's what the user wants. Focus stealing can cause an extremely disruptive
/// user experience.
pub fn gain_focus<Message>() -> Command<Message> {
    Command::single(command::Action::Window(Action::GainFocus))
}

/// Changes whether or not the window will always be on top of other windows.
pub fn change_always_on_top<Message>(on_top: bool) -> Command<Message> {
    Command::single(command::Action::Window(Action::ChangeAlwaysOnTop(on_top)))
}

/// Fetches an identifier unique to the window.
pub fn fetch_id<Message>(
    f: impl FnOnce(u64) -> Message + 'static,
) -> Command<Message> {
    Command::single(command::Action::Window(Action::FetchId(Box::new(f))))
}

/// Changes the [`Icon`] of the window.
pub fn change_icon<Message>(icon: Icon) -> Command<Message> {
    Command::single(command::Action::Window(Action::ChangeIcon(icon)))
}
