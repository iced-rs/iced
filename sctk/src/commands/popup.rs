//! Interact with the popups of your application.
use iced_native::command::{
    self,
    platform_specific::{
        self,
        wayland::{
            self,
            popup::{SctkPopupSettings, SctkPositioner},
        },
    },
};
use iced_native::window::Id as SurfaceId;
use iced_native::{command::Command, window};
pub use window::{Event, Mode};

/// <https://wayland.app/protocols/wlr-layer-shell-unstable-v1#zwlr_layer_surface_v1:request:get_popup>
/// <https://wayland.app/protocols/xdg-shell#xdg_surface:request:get_popup>
pub fn get_popup<Message>(popup: SctkPopupSettings) -> Command<Message> {
    Command::single(command::Action::PlatformSpecific(
        platform_specific::Action::Wayland(wayland::Action::Popup(
            wayland::popup::Action::Popup {
                popup,
                _phantom: Default::default(),
            },
        )),
    ))
}

/// <https://wayland.app/protocols/xdg-shell#xdg_popup:request:reposition>
pub fn reposition_popup<Message>(
    id: SurfaceId,
    positioner: SctkPositioner,
) -> Command<Message> {
    Command::single(command::Action::PlatformSpecific(
        platform_specific::Action::Wayland(wayland::Action::Popup(
            wayland::popup::Action::Reposition { id, positioner },
        )),
    ))
}

// https://wayland.app/protocols/xdg-shell#xdg_popup:request:grab
pub fn grab_popup<Message>(id: SurfaceId) -> Command<Message> {
    Command::single(command::Action::PlatformSpecific(
        platform_specific::Action::Wayland(wayland::Action::Popup(
            wayland::popup::Action::Grab { id },
        )),
    ))
}

/// <https://wayland.app/protocols/xdg-shell#xdg_popup:request:destroy>
pub fn destroy_popup<Message>(id: SurfaceId) -> Command<Message> {
    Command::single(command::Action::PlatformSpecific(
        platform_specific::Action::Wayland(wayland::Action::Popup(
            wayland::popup::Action::Destroy { id },
        )),
    ))
}
