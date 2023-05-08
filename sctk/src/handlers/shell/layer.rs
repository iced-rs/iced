use crate::{
    dpi::LogicalSize,
    event_loop::state::SctkState,
    sctk_event::{LayerSurfaceEventVariant, SctkEvent},
};
use sctk::{
    delegate_layer,
    reexports::client::Proxy,
    shell::{
        wlr_layer::{Anchor, KeyboardInteractivity, LayerShellHandler},
        WaylandSurface,
    },
};
use std::fmt::Debug;

impl<T: Debug> LayerShellHandler for SctkState<T> {
    fn closed(
        &mut self,
        _conn: &sctk::reexports::client::Connection,
        _qh: &sctk::reexports::client::QueueHandle<Self>,
        layer: &sctk::shell::wlr_layer::LayerSurface,
    ) {
        let layer = match self.layer_surfaces.iter().position(|s| {
            s.surface.wl_surface().id() == layer.wl_surface().id()
        }) {
            Some(w) => self.layer_surfaces.remove(w),
            None => return,
        };

        self.sctk_events.push(SctkEvent::LayerSurfaceEvent {
            variant: LayerSurfaceEventVariant::Done,
            id: layer.surface.wl_surface().clone(),
        })
        // TODO popup cleanup
    }

    fn configure(
        &mut self,
        _conn: &sctk::reexports::client::Connection,
        _qh: &sctk::reexports::client::QueueHandle<Self>,
        layer: &sctk::shell::wlr_layer::LayerSurface,
        mut configure: sctk::shell::wlr_layer::LayerSurfaceConfigure,
        _serial: u32,
    ) {
        let layer =
            match self.layer_surfaces.iter_mut().find(|s| {
                s.surface.wl_surface().id() == layer.wl_surface().id()
            }) {
                Some(l) => l,
                None => return,
            };

        configure.new_size.0 = if let Some(w) = layer.requested_size.0 {
            w
        } else {
            configure.new_size.0.max(1)
        };
        configure.new_size.1 = if let Some(h) = layer.requested_size.1 {
            h
        } else {
            configure.new_size.1.max(1)
        };

        layer.current_size.replace(LogicalSize::new(
            configure.new_size.0,
            configure.new_size.1,
        ));
        let first = layer.last_configure.is_none();
        layer.last_configure.replace(configure.clone());

        self.sctk_events.push(SctkEvent::LayerSurfaceEvent {
            variant: LayerSurfaceEventVariant::Configure(
                configure,
                layer.surface.wl_surface().clone(),
                first,
            ),
            id: layer.surface.wl_surface().clone(),
        });
        self.sctk_events
            .push(SctkEvent::Frame(layer.surface.wl_surface().clone()));
    }
}

delegate_layer!(@<T: 'static + Debug> SctkState<T>);

/// A request to SCTK window from Winit window.
#[derive(Debug, Clone)]
pub enum LayerSurfaceRequest {
    /// Set fullscreen.
    ///
    /// Passing `None` will set it on the current monitor.
    Size(LogicalSize<u32>),

    /// Unset fullscreen.
    UnsetFullscreen,

    /// Show cursor for the certain window or not.
    ShowCursor(bool),

    /// Set anchor
    Anchor(Anchor),

    /// Set margin
    ExclusiveZone(i32),

    /// Set margin
    Margin(u32),

    /// Passthrough mouse input to underlying windows.
    KeyboardInteractivity(KeyboardInteractivity),

    /// Redraw was requested.
    Redraw,

    /// Window should be closed.
    Close,
}
