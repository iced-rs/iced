//! Interact with the window of your application.
use std::marker::PhantomData;

use iced_native::command::platform_specific::wayland::layer_surface::IcedMargin;
use iced_native::window::Id as SurfaceId;
use iced_native::{
    command::{
        self,
        platform_specific::{
            self,
            wayland::{self, layer_surface::SctkLayerSurfaceSettings},
        },
        Command,
    },
    window,
};
pub use window::{Event, Mode};

pub use sctk::shell::layer::{Anchor, KeyboardInteractivity, Layer};

// TODO ASHLEY: maybe implement as builder that outputs a batched commands
/// <https://wayland.app/protocols/wlr-layer-shell-unstable-v1#zwlr_layer_shell_v1:request:get_layer_surface>
pub fn get_layer_surface<Message>(
    builder: SctkLayerSurfaceSettings,
) -> Command<Message> {
    Command::single(command::Action::PlatformSpecific(
        platform_specific::Action::Wayland(wayland::Action::LayerSurface(
            wayland::layer_surface::Action::LayerSurface {
                builder,
                _phantom: PhantomData::default(),
            },
        )),
    ))
}

/// <https://wayland.app/protocols/wlr-layer-shell-unstable-v1#zwlr_layer_surface_v1:request:destroy>
pub fn destroy_layer_surface<Message>(id: SurfaceId) -> Command<Message> {
    Command::single(command::Action::PlatformSpecific(
        platform_specific::Action::Wayland(wayland::Action::LayerSurface(
            wayland::layer_surface::Action::Destroy(id),
        )),
    ))
}

/// <https://wayland.app/protocols/wlr-layer-shell-unstable-v1#zwlr_layer_surface_v1:request:set_size>
pub fn set_size<Message>(
    id: SurfaceId,
    width: Option<u32>,
    height: Option<u32>,
) -> Command<Message> {
    Command::single(command::Action::PlatformSpecific(
        platform_specific::Action::Wayland(wayland::Action::LayerSurface(
            wayland::layer_surface::Action::Size { id, width, height },
        )),
    ))
}
/// <https://wayland.app/protocols/wlr-layer-shell-unstable-v1#zwlr_layer_surface_v1:request:set_anchor>
pub fn set_anchor<Message>(id: SurfaceId, anchor: Anchor) -> Command<Message> {
    Command::single(command::Action::PlatformSpecific(
        platform_specific::Action::Wayland(wayland::Action::LayerSurface(
            wayland::layer_surface::Action::Anchor { id, anchor },
        )),
    ))
}
/// <https://wayland.app/protocols/wlr-layer-shell-unstable-v1#zwlr_layer_surface_v1:request:set_exclusive_zone>
pub fn set_exclusive_zone<Message>(
    id: SurfaceId,
    zone: i32,
) -> Command<Message> {
    Command::single(command::Action::PlatformSpecific(
        platform_specific::Action::Wayland(wayland::Action::LayerSurface(
            wayland::layer_surface::Action::ExclusiveZone {
                id,
                exclusive_zone: zone,
            },
        )),
    ))
}

/// <https://wayland.app/protocols/wlr-layer-shell-unstable-v1#zwlr_layer_surface_v1:request:set_margin>
pub fn set_margin<Message>(
    id: SurfaceId,
    top: i32,
    right: i32,
    bottom: i32,
    left: i32,
) -> Command<Message> {
    Command::single(command::Action::PlatformSpecific(
        platform_specific::Action::Wayland(wayland::Action::LayerSurface(
            wayland::layer_surface::Action::Margin {
                id,
                margin: IcedMargin {
                    top,
                    right,
                    bottom,
                    left,
                },
            },
        )),
    ))
}

/// <https://wayland.app/protocols/wlr-layer-shell-unstable-v1#zwlr_layer_surface_v1:request:set_keyboard_interactivity>
pub fn set_keyboard_interactivity<Message>(
    id: SurfaceId,
    keyboard_interactivity: KeyboardInteractivity,
) -> Command<Message> {
    Command::single(command::Action::PlatformSpecific(
        platform_specific::Action::Wayland(wayland::Action::LayerSurface(
            wayland::layer_surface::Action::KeyboardInteractivity {
                id,
                keyboard_interactivity,
            },
        )),
    ))
}

/// <https://wayland.app/protocols/wlr-layer-shell-unstable-v1#zwlr_layer_surface_v1:request:set_layer>
pub fn set_layer<Message>(id: SurfaceId, layer: Layer) -> Command<Message> {
    Command::single(command::Action::PlatformSpecific(
        platform_specific::Action::Wayland(wayland::Action::LayerSurface(
            wayland::layer_surface::Action::Layer { id, layer },
        )),
    ))
}
