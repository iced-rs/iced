//! Interact with the window of your application.
use std::marker::PhantomData;

use crate::command::{self, Command};
use iced_native::command::platform_specific::{
    self,
    wayland::{self, window::SctkWindowSettings},
};
use iced_native::window;

pub use window::Action;
pub use window::{Event, Mode};

pub fn get_window<Message>(builder: SctkWindowSettings) -> Command<Message> {
    Command::single(command::Action::PlatformSpecific(
        platform_specific::Action::Wayland(wayland::Action::Window(
            wayland::window::Action::Window {
                builder,
                _phantom: PhantomData::default(),
            },
        )),
    ))
}

// TODO Ashley refactor to use regular window events maybe...
/// close the window
pub fn close_window<Message>(id: window::Id) -> Command<Message> {
    Command::single(command::Action::PlatformSpecific(
        platform_specific::Action::Wayland(wayland::Action::Window(
            wayland::window::Action::Destroy(id),
        )),
    ))
}

/// Resizes the window to the given logical dimensions.
pub fn resize_window<Message>(
    id: window::Id,
    width: u32,
    height: u32,
) -> Command<Message> {
    Command::single(command::Action::PlatformSpecific(
        platform_specific::Action::Wayland(wayland::Action::Window(
            wayland::window::Action::Size { id, width, height },
        )),
    ))
}

/// Sets the [`Mode`] of the window.
pub fn set_mode_window<Message>(
    id: window::Id,
    mode: Mode,
) -> Command<Message> {
    Command::single(command::Action::Window(id, Action::SetMode(mode)))
}

/// Fetches the current [`Mode`] of the window.
pub fn fetch_mode_window<Message>(
    id: window::Id,
    f: impl FnOnce(Mode) -> Message + 'static,
) -> Command<Message> {
    Command::single(command::Action::Window(id, Action::FetchMode(Box::new(f))))
}
