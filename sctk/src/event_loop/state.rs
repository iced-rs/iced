use std::{
    collections::HashMap,
    fmt::{Debug, Formatter},
};

use crate::{
    application::Event,
    dpi::LogicalSize,
    sctk_event::{SctkEvent, SurfaceCompositorUpdate},
};

use iced_runtime::{
    command::platform_specific::{
        self,
        wayland::{
            data_device::DataFromMimeType,
            layer_surface::{IcedMargin, IcedOutput, SctkLayerSurfaceSettings},
            popup::SctkPopupSettings,
            window::SctkWindowSettings,
        },
    },
    keyboard::Modifiers,
    window,
};
use sctk::{
    compositor::CompositorState,
    data_device_manager::{
        data_device::DataDevice,
        data_offer::{DragOffer, SelectionOffer},
        data_source::{CopyPasteSource, DragSource},
        DataDeviceManagerState, WritePipe,
    },
    error::GlobalError,
    output::OutputState,
    reexports::{
        calloop::{LoopHandle, RegistrationToken},
        client::{
            backend::ObjectId,
            protocol::{
                wl_keyboard::WlKeyboard,
                wl_output::WlOutput,
                wl_pointer::WlPointer,
                wl_seat::WlSeat,
                wl_surface::{self, WlSurface},
                wl_touch::WlTouch,
            },
            Connection, QueueHandle,
        },
    },
    registry::RegistryState,
    seat::{keyboard::KeyEvent, SeatState},
    shell::{
        wlr_layer::{
            Anchor, KeyboardInteractivity, Layer, LayerShell, LayerSurface,
            LayerSurfaceConfigure,
        },
        xdg::{
            popup::{Popup, PopupConfigure},
            window::{Window, WindowConfigure, WindowDecorations},
            XdgPositioner, XdgShell, XdgSurface,
        },
        WaylandSurface,
    },
    shm::{multi::MultiPool, Shm},
};

#[derive(Debug, Clone)]
pub(crate) struct SctkSeat {
    pub(crate) seat: WlSeat,
    pub(crate) kbd: Option<WlKeyboard>,
    pub(crate) kbd_focus: Option<WlSurface>,
    pub(crate) last_kbd_press: Option<(KeyEvent, u32)>,
    pub(crate) ptr: Option<WlPointer>,
    pub(crate) ptr_focus: Option<WlSurface>,
    pub(crate) last_ptr_press: Option<(u32, u32, u32)>, // (time, button, serial)
    pub(crate) touch: Option<WlTouch>,
    pub(crate) modifiers: Modifiers,
    pub(crate) data_device: DataDevice,
}

#[derive(Debug, Clone)]
pub struct SctkWindow<T> {
    pub(crate) id: window::Id,
    pub(crate) window: Window,
    pub(crate) requested_size: Option<(u32, u32)>,
    pub(crate) current_size: Option<(u32, u32)>,
    pub(crate) last_configure: Option<WindowConfigure>,
    /// Requests that SCTK window should perform.
    pub(crate) pending_requests:
        Vec<platform_specific::wayland::window::Action<T>>,
}

#[derive(Debug, Clone)]
pub struct SctkLayerSurface<T> {
    pub(crate) id: window::Id,
    pub(crate) surface: LayerSurface,
    pub(crate) requested_size: (Option<u32>, Option<u32>),
    pub(crate) current_size: Option<LogicalSize<u32>>,
    pub(crate) layer: Layer,
    pub(crate) anchor: Anchor,
    pub(crate) keyboard_interactivity: KeyboardInteractivity,
    pub(crate) margin: IcedMargin,
    pub(crate) exclusive_zone: i32,
    pub(crate) last_configure: Option<LayerSurfaceConfigure>,
    pub(crate) pending_requests:
        Vec<platform_specific::wayland::layer_surface::Action<T>>,
}

#[derive(Debug, Clone)]
pub enum SctkSurface {
    LayerSurface(WlSurface),
    Window(WlSurface),
    Popup(WlSurface),
}

impl SctkSurface {
    pub fn wl_surface(&self) -> &WlSurface {
        match self {
            SctkSurface::LayerSurface(s)
            | SctkSurface::Window(s)
            | SctkSurface::Popup(s) => s,
        }
    }
}

#[derive(Debug)]
pub struct SctkPopup<T> {
    pub(crate) popup: Popup,
    pub(crate) last_configure: Option<PopupConfigure>,
    // pub(crate) positioner: XdgPositioner,
    pub(crate) pending_requests:
        Vec<platform_specific::wayland::popup::Action<T>>,
    pub(crate) data: SctkPopupData,
}

pub struct Dnd<T> {
    pub(crate) origin_id: window::Id,
    pub(crate) origin: WlSurface,
    pub(crate) source: Option<(DragSource, Box<dyn DataFromMimeType>)>,
    pub(crate) icon_surface: Option<(WlSurface, window::Id)>,
    pub(crate) pending_requests:
        Vec<platform_specific::wayland::data_device::Action<T>>,
    pub(crate) pipe: Option<WritePipe>,
    pub(crate) cur_write: Option<(Vec<u8>, usize, RegistrationToken)>,
}

impl<T> Debug for Dnd<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Dnd")
            .field(&self.origin_id)
            .field(&self.origin)
            .field(&self.icon_surface)
            .field(&self.pending_requests)
            .field(&self.pipe)
            .field(&self.cur_write)
            .finish()
    }
}

#[derive(Debug)]
pub struct SctkSelectionOffer {
    pub(crate) offer: SelectionOffer,
    pub(crate) cur_read: Option<(String, Vec<u8>, RegistrationToken)>,
}

#[derive(Debug)]
pub struct SctkDragOffer {
    pub(crate) dropped: bool,
    pub(crate) offer: DragOffer,
    pub(crate) cur_read: Option<(String, Vec<u8>, RegistrationToken)>,
}

#[derive(Debug)]
pub struct SctkPopupData {
    pub(crate) id: window::Id,
    pub(crate) parent: SctkSurface,
    pub(crate) toplevel: WlSurface,
    pub(crate) positioner: XdgPositioner,
}

pub struct SctkCopyPasteSource {
    pub accepted_mime_types: Vec<String>,
    pub source: CopyPasteSource,
    pub data: Box<dyn DataFromMimeType>,
    pub(crate) pipe: Option<WritePipe>,
    pub(crate) cur_write: Option<(Vec<u8>, usize, RegistrationToken)>,
}

impl Debug for SctkCopyPasteSource {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("SctkCopyPasteSource")
            .field(&self.accepted_mime_types)
            .field(&self.source)
            .field(&self.pipe)
            .field(&self.cur_write)
            .finish()
    }
}

/// Wrapper to carry sctk state.
pub struct SctkState<T> {
    /// the cursor wl_surface
    pub(crate) cursor_surface: Option<wl_surface::WlSurface>,
    /// a memory pool
    pub(crate) multipool: Option<MultiPool<WlSurface>>,

    // all present outputs
    pub(crate) outputs: Vec<WlOutput>,
    // though (for now) only one seat will be active in an iced application at a time, all ought to be tracked
    // Active seat is the first seat in the list
    pub(crate) seats: Vec<SctkSeat>,
    // Windows / Surfaces
    /// Window list containing all SCTK windows. Since those windows aren't allowed
    /// to be sent to other threads, they live on the event loop's thread
    /// and requests from winit's windows are being forwarded to them either via
    /// `WindowUpdate` or buffer on the associated with it `WindowHandle`.
    pub(crate) windows: Vec<SctkWindow<T>>,
    pub(crate) layer_surfaces: Vec<SctkLayerSurface<T>>,
    pub(crate) popups: Vec<SctkPopup<T>>,
    pub(crate) dnd_source: Option<Dnd<T>>,
    pub(crate) kbd_focus: Option<WlSurface>,

    /// Window updates, which are coming from SCTK or the compositor, which require
    /// calling back to the sctk's downstream. They are handled right in the event loop,
    /// unlike the ones coming from buffers on the `WindowHandle`'s.
    pub popup_compositor_updates: HashMap<ObjectId, SurfaceCompositorUpdate>,
    /// Window updates, which are coming from SCTK or the compositor, which require
    /// calling back to the sctk's downstream. They are handled right in the event loop,
    /// unlike the ones coming from buffers on the `WindowHandle`'s.
    pub window_compositor_updates: HashMap<ObjectId, SurfaceCompositorUpdate>,
    /// Layer Surface updates, which are coming from SCTK or the compositor, which require
    /// calling back to the sctk's downstream. They are handled right in the event loop,
    /// unlike the ones coming from buffers on the `WindowHandle`'s.
    pub layer_surface_compositor_updates:
        HashMap<ObjectId, SurfaceCompositorUpdate>,

    /// data data_device
    pub(crate) selection_source: Option<SctkCopyPasteSource>,
    pub(crate) dnd_offer: Option<SctkDragOffer>,
    pub(crate) selection_offer: Option<SctkSelectionOffer>,
    pub(crate) accept_counter: u32,
    /// A sink for window and device events that is being filled during dispatching
    /// event loop and forwarded downstream afterwards.
    pub(crate) sctk_events: Vec<SctkEvent>,

    /// pending user events
    pub pending_user_events: Vec<Event<T>>,

    // handles
    pub(crate) queue_handle: QueueHandle<Self>,
    pub(crate) loop_handle: LoopHandle<'static, Self>,

    // sctk state objects
    pub(crate) registry_state: RegistryState,
    pub(crate) seat_state: SeatState,
    pub(crate) output_state: OutputState,
    pub(crate) compositor_state: CompositorState,
    pub(crate) shm_state: Shm,
    pub(crate) xdg_shell_state: XdgShell,
    pub(crate) layer_shell: Option<LayerShell>,
    pub(crate) data_device_manager_state: DataDeviceManagerState,
    pub(crate) connection: Connection,
    pub(crate) token_ctr: u32,
}

/// An error that occurred while running an application.
#[derive(Debug, thiserror::Error)]
pub enum PopupCreationError {
    /// Positioner creation failed
    #[error("Positioner creation failed")]
    PositionerCreationFailed(GlobalError),

    /// The specified parent is missing
    #[error("The specified parent is missing")]
    ParentMissing,

    /// The specified size is missing
    #[error("The specified size is missing")]
    SizeMissing,

    /// Popup creation failed
    #[error("Popup creation failed")]
    PopupCreationFailed(GlobalError),
}

/// An error that occurred while running an application.
#[derive(Debug, thiserror::Error)]
pub enum LayerSurfaceCreationError {
    /// Layer shell is not supported by the compositor
    #[error("Layer shell is not supported by the compositor")]
    LayerShellNotSupported,

    /// WlSurface creation failed
    #[error("WlSurface creation failed")]
    WlSurfaceCreationFailed(GlobalError),

    /// LayerSurface creation failed
    #[error("Layer Surface creation failed")]
    LayerSurfaceCreationFailed(GlobalError),
}

/// An error that occurred while starting a drag and drop operation.
#[derive(Debug, thiserror::Error)]
pub enum DndStartError {}

impl<T> SctkState<T>
where
    T: 'static + Debug,
{
    pub fn get_popup(
        &mut self,
        settings: SctkPopupSettings,
    ) -> Result<(window::Id, WlSurface, WlSurface, WlSurface), PopupCreationError>
    {
        let (parent, toplevel) = if let Some(parent) =
            self.layer_surfaces.iter().find(|l| l.id == settings.parent)
        {
            (
                SctkSurface::LayerSurface(parent.surface.wl_surface().clone()),
                parent.surface.wl_surface().clone(),
            )
        } else if let Some(parent) =
            self.windows.iter().find(|w| w.id == settings.parent)
        {
            (
                SctkSurface::Window(parent.window.wl_surface().clone()),
                parent.window.wl_surface().clone(),
            )
        } else if let Some(i) = self
            .popups
            .iter()
            .position(|p| p.data.id == settings.parent)
        {
            let parent = &self.popups[i];
            (
                SctkSurface::Popup(parent.popup.wl_surface().clone()),
                parent.data.toplevel.clone(),
            )
        } else {
            return Err(PopupCreationError::ParentMissing);
        };

        let size = if settings.positioner.size.is_none() {
            return Err(PopupCreationError::SizeMissing);
        } else {
            settings.positioner.size.unwrap()
        };

        let positioner = XdgPositioner::new(&self.xdg_shell_state)
            .map_err(PopupCreationError::PositionerCreationFailed)?;
        positioner.set_anchor(settings.positioner.anchor);
        positioner.set_anchor_rect(
            settings.positioner.anchor_rect.x,
            settings.positioner.anchor_rect.y,
            settings.positioner.anchor_rect.width,
            settings.positioner.anchor_rect.height,
        );
        positioner.set_constraint_adjustment(
            settings.positioner.constraint_adjustment,
        );
        positioner.set_gravity(settings.positioner.gravity);
        positioner.set_offset(
            settings.positioner.offset.0,
            settings.positioner.offset.1,
        );
        if settings.positioner.reactive {
            positioner.set_reactive();
        }
        positioner.set_size(size.0 as i32, size.1 as i32);

        let grab = settings.grab;

        let wl_surface =
            self.compositor_state.create_surface(&self.queue_handle);

        let (toplevel, popup) = match &parent {
            SctkSurface::LayerSurface(parent) => {
                let parent_layer_surface = self
                    .layer_surfaces
                    .iter()
                    .find(|w| w.surface.wl_surface() == parent)
                    .unwrap();
                let popup = Popup::from_surface(
                    None,
                    &positioner,
                    &self.queue_handle,
                    wl_surface.clone(),
                    &self.xdg_shell_state,
                )
                .map_err(PopupCreationError::PopupCreationFailed)?;
                parent_layer_surface.surface.get_popup(popup.xdg_popup());
                (parent_layer_surface.surface.wl_surface(), popup)
            }
            SctkSurface::Window(parent) => {
                let parent_window = self
                    .windows
                    .iter()
                    .find(|w| w.window.wl_surface() == parent)
                    .unwrap();
                (
                    parent_window.window.wl_surface(),
                    Popup::from_surface(
                        Some(parent_window.window.xdg_surface()),
                        &positioner,
                        &self.queue_handle,
                        wl_surface.clone(),
                        &self.xdg_shell_state,
                    )
                    .map_err(PopupCreationError::PopupCreationFailed)?,
                )
            }
            SctkSurface::Popup(parent) => {
                let parent_xdg = self
                    .windows
                    .iter()
                    .find_map(|w| {
                        if w.window.wl_surface() == parent {
                            Some(w.window.xdg_surface())
                        } else {
                            None
                        }
                    })
                    .unwrap();

                (
                    &toplevel,
                    Popup::from_surface(
                        Some(parent_xdg),
                        &positioner,
                        &self.queue_handle,
                        wl_surface.clone(),
                        &self.xdg_shell_state,
                    )
                    .map_err(PopupCreationError::PopupCreationFailed)?,
                )
            }
        };
        if grab {
            if let Some(s) = self.seats.first() {
                popup.xdg_popup().grab(
                    &s.seat,
                    s.last_ptr_press.map(|p| p.2).unwrap_or_else(|| {
                        s.last_kbd_press
                            .as_ref()
                            .map(|p| p.1)
                            .unwrap_or_default()
                    }),
                )
            }
        }
        wl_surface.commit();
        self.popups.push(SctkPopup {
            popup: popup.clone(),
            data: SctkPopupData {
                id: settings.id,
                parent: parent.clone(),
                toplevel: toplevel.clone(),
                positioner,
            },
            last_configure: None,
            pending_requests: Default::default(),
        });

        Ok((
            settings.id,
            parent.wl_surface().clone(),
            toplevel.clone(),
            popup.wl_surface().clone(),
        ))
    }

    pub fn get_window(
        &mut self,
        settings: SctkWindowSettings,
    ) -> (window::Id, WlSurface) {
        let SctkWindowSettings {
            size,
            decorations,
            transparent,
            window_id,
            app_id,
            title,
            parent,
            size_limits,
            ..
        } = settings;
        // TODO Ashley: set window as opaque if transparency is false
        // TODO Ashley: set icon
        // TODO Ashley: save settings for window
        // TODO Ashley: decorations
        let wl_surface =
            self.compositor_state.create_surface(&self.queue_handle);
        let decorations = if decorations {
            WindowDecorations::RequestServer
        } else {
            WindowDecorations::RequestClient
        };
        let window = self.xdg_shell_state.create_window(
            wl_surface.clone(),
            decorations,
            &self.queue_handle,
        );
        if let Some(app_id) = app_id {
            window.set_app_id(app_id);
        }
        let min_size = size_limits.min();
        let min_size =
            if min_size.width as i32 <= 0 || min_size.height as i32 <= 0 {
                None
            } else {
                Some((min_size.width as u32, min_size.height as u32))
            };
        let max_size = size_limits.max();
        let max_size =
            if max_size.width as i32 <= 0 || max_size.height as i32 <= 0 {
                None
            } else {
                Some((max_size.width as u32, max_size.height as u32))
            };
        window.set_min_size(min_size);
        window.set_max_size(max_size);
        if let Some(title) = title {
            window.set_title(title);
        }
        // if let Some(parent) = parent.and_then(|p| self.windows.iter().find(|w| w.window.wl_surface().id() == p)) {
        //     window.set_parent(Some(&parent.window));
        // }
        window.commit();
        window.xdg_surface().set_window_geometry(
            0,
            0,
            size.0 as i32,
            size.1 as i32,
        );
        window.wl_surface().commit();
        self.windows.push(SctkWindow {
            id: window_id,
            window,
            requested_size: Some(size),
            current_size: Some((1, 1)),
            last_configure: None,
            pending_requests: Vec::new(),
        });
        (window_id, wl_surface)
    }

    pub fn get_layer_surface(
        &mut self,
        SctkLayerSurfaceSettings {
            id,
            layer,
            keyboard_interactivity,
            pointer_interactivity,
            anchor,
            output,
            namespace,
            margin,
            size,
            exclusive_zone,
            ..
        }: SctkLayerSurfaceSettings,
    ) -> Result<(window::Id, WlSurface), LayerSurfaceCreationError> {
        let wl_output = match output {
            IcedOutput::All => None, // TODO
            IcedOutput::Active => None,
            IcedOutput::Output(output) => Some(output),
        };

        let layer_shell = self
            .layer_shell
            .as_ref()
            .ok_or(LayerSurfaceCreationError::LayerShellNotSupported)?;
        let wl_surface =
            self.compositor_state.create_surface(&self.queue_handle);
        let mut size = size.unwrap();
        if anchor.contains(Anchor::BOTTOM.union(Anchor::TOP)) {
            size.1 = None;
        }
        if anchor.contains(Anchor::LEFT.union(Anchor::RIGHT)) {
            size.0 = None;
        }
        let layer_surface = layer_shell.create_layer_surface(
            &self.queue_handle,
            wl_surface.clone(),
            layer,
            Some(namespace),
            wl_output.as_ref(),
        );
        layer_surface.set_anchor(anchor);
        layer_surface.set_keyboard_interactivity(keyboard_interactivity);
        layer_surface.set_margin(
            margin.top,
            margin.right,
            margin.bottom,
            margin.left,
        );
        layer_surface
            .set_size(size.0.unwrap_or_default(), size.1.unwrap_or_default());
        layer_surface.set_exclusive_zone(exclusive_zone);
        if !pointer_interactivity {
            layer_surface.set_input_region(None);
        }
        layer_surface.commit();
        self.layer_surfaces.push(SctkLayerSurface {
            id,
            surface: layer_surface,
            requested_size: size,
            current_size: None,
            layer,
            // builder needs to be refactored such that these fields are accessible
            anchor,
            keyboard_interactivity,
            margin,
            exclusive_zone,
            last_configure: None,
            pending_requests: Vec::new(),
        });
        Ok((id, wl_surface))
    }
}
