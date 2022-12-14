// SPDX-License-Identifier: MPL-2.0-only
use sctk::{
    compositor::CompositorHandler,
    delegate_compositor,
    reexports::client::{protocol::wl_surface, Connection, Proxy, QueueHandle},
};
use std::fmt::Debug;

use crate::{event_loop::state::SctkState, sctk_event::SctkEvent};

impl<T: Debug> CompositorHandler for SctkState<T> {
    fn scale_factor_changed(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        surface: &wl_surface::WlSurface,
        new_factor: i32,
    ) {
        if let Some(w) = self
            .windows
            .iter()
            .find(|w| w.window.wl_surface().id() == surface.id())
        {
            if let Some(e) =
                self.window_compositor_updates.get_mut(&surface.id())
            {
                e.scale_factor = Some(new_factor)
            }
        }
        if let Some(w) = self
            .layer_surfaces
            .iter()
            .find(|w| w.surface.wl_surface().id() == surface.id())
        {
            if let Some(e) =
                self.layer_surface_compositor_updates.get_mut(&surface.id())
            {
                e.scale_factor = Some(new_factor)
            }
        }
        if let Some(w) = self
            .popups
            .iter()
            .find(|w| w.popup.wl_surface().id() == surface.id())
        {
            if let Some(e) =
                self.popup_compositor_updates.get_mut(&surface.id())
            {
                e.scale_factor = Some(new_factor)
            }
        }
    }

    fn frame(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        surface: &wl_surface::WlSurface,
        _time: u32,
    ) {
        self.sctk_events.push(SctkEvent::Draw(surface.clone()));
    }
}

delegate_compositor!(@<T: 'static + Debug> SctkState<T>);
