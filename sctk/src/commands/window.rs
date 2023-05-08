//! Interact with the window of your application.
use std::marker::PhantomData;

use iced_runtime::{
    command::{
        self,
        platform_specific::{
            self,
            wayland::{self, window::SctkWindowSettings},
        },
    },
    core::window::Mode,
    window, Command,
};

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

pub fn start_drag_window<Message>(id: window::Id) -> Command<Message> {
    Command::single(command::Action::PlatformSpecific(
        platform_specific::Action::Wayland(wayland::Action::Window(
            wayland::window::Action::InteractiveMove { id },
        )),
    ))
}

pub fn toggle_maximize<Message>(id: window::Id) -> Command<Message> {
    Command::single(command::Action::PlatformSpecific(
        platform_specific::Action::Wayland(wayland::Action::Window(
            wayland::window::Action::ToggleMaximized { id },
        )),
    ))
}

pub fn set_app_id_window<Message>(
    id: window::Id,
    app_id: String,
) -> Command<Message> {
    Command::single(command::Action::PlatformSpecific(
        platform_specific::Action::Wayland(wayland::Action::Window(
            wayland::window::Action::AppId { id, app_id },
        )),
    ))
}

/// Sets the [`Mode`] of the window.
pub fn set_mode_window<Message>(
    id: window::Id,
    mode: Mode,
) -> Command<Message> {
    Command::single(command::Action::PlatformSpecific(
        platform_specific::Action::Wayland(wayland::Action::Window(
            wayland::window::Action::Mode(id, mode),
        )),
    ))
}
