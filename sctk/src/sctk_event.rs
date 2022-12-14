use std::{collections::HashMap, time::Instant};

use crate::{
    application::SurfaceIdWrapper,
    conversion::{
        keysym_to_vkey, modifiers_to_native, pointer_axis_to_native,
        pointer_button_to_native,
    },
    dpi::{LogicalSize, PhysicalSize},
};
use iced_graphics::Point;
use iced_native::{
    event::{
        wayland::{self, LayerEvent, PopupEvent},
        PlatformSpecific,
    },
    keyboard::{self, KeyCode},
    mouse,
    window::{self, Id as SurfaceId},
};
use sctk::{
    output::OutputInfo,
    reexports::client::{
        backend::ObjectId,
        protocol::{
            wl_keyboard::WlKeyboard,
            wl_output::WlOutput,
            wl_pointer::WlPointer,
            wl_seat::{self, WlSeat},
            wl_surface::WlSurface,
        },
        Proxy,
    },
    seat::{
        keyboard::{KeyEvent, Modifiers},
        pointer::{PointerEvent, PointerEventKind},
        Capability,
    },
    shell::{
        layer::LayerSurfaceConfigure,
        xdg::{popup::PopupConfigure, window::WindowConfigure},
    },
};

#[derive(Debug, Clone)]
pub enum IcedSctkEvent<T> {
    /// Emitted when new events arrive from the OS to be processed.
    ///
    /// This event type is useful as a place to put code that should be done before you start
    /// processing events, such as updating frame timing information for benchmarking or checking
    /// the [`StartCause`][crate::event::StartCause] to see if a timer set by
    /// [`ControlFlow::WaitUntil`](crate::event_loop::ControlFlow::WaitUntil) has elapsed.
    NewEvents(StartCause),

    /// Any user event from iced
    UserEvent(T),
    /// An event produced by sctk
    SctkEvent(SctkEvent),

    /// Emitted when all of the event loop's input events have been processed and redraw processing
    /// is about to begin.
    ///
    /// This event is useful as a place to put your code that should be run after all
    /// state-changing events have been handled and you want to do stuff (updating state, performing
    /// calculations, etc) that happens as the "main body" of your event loop. If your program only draws
    /// graphics when something changes, it's usually better to do it in response to
    /// [`Event::RedrawRequested`](crate::event::Event::RedrawRequested), which gets emitted
    /// immediately after this event. Programs that draw graphics continuously, like most games,
    /// can render here unconditionally for simplicity.
    MainEventsCleared,

    /// Emitted after [`MainEventsCleared`] when a window should be redrawn.
    ///
    /// This gets triggered in two scenarios:
    /// - The OS has performed an operation that's invalidated the window's contents (such as
    ///   resizing the window).
    /// - The application has explicitly requested a redraw via [`Window::request_redraw`].
    ///
    /// During each iteration of the event loop, Winit will aggregate duplicate redraw requests
    /// into a single event, to help avoid duplicating rendering work.
    ///
    /// Mainly of interest to applications with mostly-static graphics that avoid redrawing unless
    /// something changes, like most non-game GUIs.
    ///
    /// [`MainEventsCleared`]: Self::MainEventsCleared
    RedrawRequested(ObjectId),

    /// Emitted after all [`RedrawRequested`] events have been processed and control flow is about to
    /// be taken away from the program. If there are no `RedrawRequested` events, it is emitted
    /// immediately after `MainEventsCleared`.
    ///
    /// This event is useful for doing any cleanup or bookkeeping work after all the rendering
    /// tasks have been completed.
    ///
    /// [`RedrawRequested`]: Self::RedrawRequested
    RedrawEventsCleared,

    /// Emitted when the event loop is being shut down.
    ///
    /// This is irreversible - if this event is emitted, it is guaranteed to be the last event that
    /// gets emitted. You generally want to treat this as an "do on quit" event.
    LoopDestroyed,
}

#[derive(Debug, Clone)]
pub enum SctkEvent {
    //
    // Input events
    //
    SeatEvent {
        variant: SeatEventVariant,
        id: WlSeat,
    },
    PointerEvent {
        variant: PointerEvent,
        ptr_id: WlPointer,
        seat_id: WlSeat,
    },
    KeyboardEvent {
        variant: KeyboardEventVariant,
        kbd_id: WlKeyboard,
        seat_id: WlSeat,
    },
    // TODO data device & touch

    //
    // Surface Events
    //
    WindowEvent {
        variant: WindowEventVariant,
        id: WlSurface,
    },
    LayerSurfaceEvent {
        variant: LayerSurfaceEventVariant,
        id: WlSurface,
    },
    PopupEvent {
        variant: PopupEventVariant,
        /// this may be the Id of a window or layer surface
        toplevel_id: WlSurface,
        /// this may be any SurfaceId
        parent_id: WlSurface,
        /// the id of this popup
        id: WlSurface,
    },

    //
    // output events
    //
    NewOutput {
        id: WlOutput,
        info: Option<OutputInfo>,
    },
    UpdateOutput {
        id: WlOutput,
        info: OutputInfo,
    },
    RemovedOutput(ObjectId),

    //
    // compositor events
    //
    Draw(WlSurface),
    ScaleFactorChanged {
        factor: f64,
        id: WlOutput,
        inner_size: PhysicalSize<u32>,
    },
}

#[derive(Debug, Clone)]
pub enum SeatEventVariant {
    New,
    Remove,
    NewCapability(Capability, ObjectId),
    RemoveCapability(Capability, ObjectId),
}

#[derive(Debug, Clone)]
pub enum KeyboardEventVariant {
    Leave(WlSurface),
    Enter(WlSurface),
    Press(KeyEvent),
    Repeat(KeyEvent),
    Release(KeyEvent),
    Modifiers(Modifiers),
}

#[derive(Debug, Clone)]
pub enum WindowEventVariant {
    Created(ObjectId, SurfaceId),
    /// <https://wayland.app/protocols/xdg-shell#xdg_toplevel:event:close>
    Close,
    /// <https://wayland.app/protocols/xdg-shell#xdg_toplevel:event:wm_capabilities>
    WmCapabilities(Vec<u32>),
    /// <https://wayland.app/protocols/xdg-shell#xdg_toplevel:event:configure_bounds>
    ConfigureBounds {
        width: u32,
        height: u32,
    },
    /// <https://wayland.app/protocols/xdg-shell#xdg_toplevel:event:configure>
    Configure(WindowConfigure, WlSurface, bool),
}

#[derive(Debug, Clone)]
pub enum PopupEventVariant {
    Created(ObjectId, SurfaceId),
    /// <https://wayland.app/protocols/xdg-shell#xdg_popup:event:popup_done>
    Done,
    /// <https://wayland.app/protocols/xdg-shell#xdg_toplevel:event:wm_capabilities>
    WmCapabilities(Vec<u32>),
    /// <https://wayland.app/protocols/xdg-shell#xdg_popup:event:configure>
    Configure(PopupConfigure, WlSurface, bool),
    /// <https://wayland.app/protocols/xdg-shell#xdg_popup:event:repositioned>
    RepositionionedPopup {
        token: u32,
    },
}

#[derive(Debug, Clone)]
pub enum LayerSurfaceEventVariant {
    /// sent after creation of the layer surface
    Created(ObjectId, SurfaceId),
    /// <https://wayland.app/protocols/wlr-layer-shell-unstable-v1#zwlr_layer_surface_v1:event:closed>
    Done,
    /// <https://wayland.app/protocols/wlr-layer-shell-unstable-v1#zwlr_layer_surface_v1:event:configure>
    Configure(LayerSurfaceConfigure, WlSurface, bool),
}

/// Describes the reason the event loop is resuming.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StartCause {
    /// Sent if the time specified by [`ControlFlow::WaitUntil`] has been reached. Contains the
    /// moment the timeout was requested and the requested resume time. The actual resume time is
    /// guaranteed to be equal to or after the requested resume time.
    ///
    /// [`ControlFlow::WaitUntil`]: crate::event_loop::ControlFlow::WaitUntil
    ResumeTimeReached {
        start: Instant,
        requested_resume: Instant,
    },

    /// Sent if the OS has new events to send to the window, after a wait was requested. Contains
    /// the moment the wait was requested and the resume time, if requested.
    WaitCancelled {
        start: Instant,
        requested_resume: Option<Instant>,
    },

    /// Sent if the event loop is being resumed after the loop's control flow was set to
    /// [`ControlFlow::Poll`].
    ///
    /// [`ControlFlow::Poll`]: crate::event_loop::ControlFlow::Poll
    Poll,

    /// Sent once, immediately after `run` is called. Indicates that the loop was just initialized.
    Init,
}

/// Pending update to a window requested by the user.
#[derive(Default, Debug, Clone, Copy)]
pub struct SurfaceUserRequest {
    /// Whether `redraw` was requested.
    pub redraw_requested: bool,

    /// Wether the frame should be refreshed.
    pub refresh_frame: bool,
}

// The window update coming from the compositor.
#[derive(Default, Debug, Clone)]
pub struct SurfaceCompositorUpdate {
    /// New window configure.
    pub configure: Option<WindowConfigure>,

    /// first
    pub first: bool,

    /// New scale factor.
    pub scale_factor: Option<i32>,

    /// Close the window.
    pub close_window: bool,
}

impl SctkEvent {
    pub fn to_native(
        self,
        modifiers: &mut Modifiers,
        surface_ids: &HashMap<ObjectId, SurfaceIdWrapper>,
        destroyed_surface_ids: &HashMap<ObjectId, SurfaceIdWrapper>,
    ) -> Vec<iced_native::Event> {
        match self {
            // TODO Ashley: Platform specific multi-seat events?
            SctkEvent::SeatEvent { .. } => Default::default(),
            SctkEvent::PointerEvent { variant, .. } => match variant.kind {
                PointerEventKind::Enter { .. } => {
                    vec![iced_native::Event::Mouse(mouse::Event::CursorEntered)]
                }
                PointerEventKind::Leave { .. } => {
                    vec![iced_native::Event::Mouse(mouse::Event::CursorLeft)]
                }
                PointerEventKind::Motion { .. } => {
                    vec![iced_native::Event::Mouse(mouse::Event::CursorMoved {
                        position: Point::new(
                            variant.position.0 as f32,
                            variant.position.1 as f32,
                        ),
                    })]
                }
                PointerEventKind::Press {
                    time: _,
                    button,
                    serial: _,
                } => pointer_button_to_native(button)
                    .map(|b| {
                        iced_native::Event::Mouse(mouse::Event::ButtonPressed(
                            b,
                        ))
                    })
                    .into_iter()
                    .collect(), // TODO Ashley: conversion
                PointerEventKind::Release {
                    time: _,
                    button,
                    serial: _,
                } => pointer_button_to_native(button)
                    .map(|b| {
                        iced_native::Event::Mouse(mouse::Event::ButtonReleased(
                            b,
                        ))
                    })
                    .into_iter()
                    .collect(), // TODO Ashley: conversion
                PointerEventKind::Axis {
                    time: _,
                    horizontal,
                    vertical,
                    source,
                } => pointer_axis_to_native(source, horizontal, vertical)
                    .map(|a| {
                        iced_native::Event::Mouse(mouse::Event::WheelScrolled {
                            delta: a,
                        })
                    })
                    .into_iter()
                    .collect(), // TODO Ashley: conversion
            },
            SctkEvent::KeyboardEvent {
                variant,
                kbd_id: _,
                seat_id,
            } => match variant {
                KeyboardEventVariant::Leave(surface) => surface_ids
                    .get(&surface.id())
                    .map(|id| match id {
                        SurfaceIdWrapper::LayerSurface(_id) => {
                            iced_native::Event::PlatformSpecific(
                                PlatformSpecific::Wayland(
                                    wayland::Event::Layer(
                                        LayerEvent::Unfocused,
                                        surface,
                                        id.inner(),
                                    ),
                                ),
                            )
                        }
                        SurfaceIdWrapper::Window(id) => {
                            iced_native::Event::Window(
                                *id,
                                window::Event::Unfocused,
                            )
                        }
                        SurfaceIdWrapper::Popup(_id) => {
                            iced_native::Event::PlatformSpecific(
                                PlatformSpecific::Wayland(
                                    wayland::Event::Popup(
                                        PopupEvent::Unfocused,
                                        surface,
                                        id.inner(),
                                    ),
                                ),
                            )
                        }
                    })
                    .into_iter()
                    .chain([iced_native::Event::PlatformSpecific(
                        PlatformSpecific::Wayland(wayland::Event::Seat(
                            wayland::SeatEvent::Leave,
                            seat_id,
                        )),
                    )])
                    .collect(),
                KeyboardEventVariant::Enter(surface) => surface_ids
                    .get(&surface.id())
                    .map(|id| match id {
                        SurfaceIdWrapper::LayerSurface(_id) => {
                            iced_native::Event::PlatformSpecific(
                                PlatformSpecific::Wayland(
                                    wayland::Event::Layer(
                                        LayerEvent::Focused,
                                        surface,
                                        id.inner(),
                                    ),
                                ),
                            )
                        }
                        SurfaceIdWrapper::Window(id) => {
                            iced_native::Event::Window(
                                *id,
                                window::Event::Focused,
                            )
                        }
                        SurfaceIdWrapper::Popup(_id) => {
                            iced_native::Event::PlatformSpecific(
                                PlatformSpecific::Wayland(
                                    wayland::Event::Popup(
                                        PopupEvent::Focused,
                                        surface,
                                        id.inner(),
                                    ),
                                ),
                            )
                        }
                    })
                    .into_iter()
                    .chain([iced_native::Event::PlatformSpecific(
                        PlatformSpecific::Wayland(wayland::Event::Seat(
                            wayland::SeatEvent::Enter,
                            seat_id,
                        )),
                    )])
                    .collect(),
                KeyboardEventVariant::Press(ke) => {
                    let mut skip_char = false;

                    let mut events: Vec<_> = keysym_to_vkey(ke.keysym)
                        .map(|k| {
                            if k == KeyCode::Backspace {
                                skip_char = true;
                            }
                            iced_native::Event::Keyboard(
                                keyboard::Event::KeyPressed {
                                    key_code: k,
                                    modifiers: modifiers_to_native(*modifiers),
                                },
                            )
                        })
                        .into_iter()
                        .collect();
                    if !skip_char {
                        if let Some(s) = ke.utf8 {
                            let mut chars = s
                                .chars()
                                .map(|c| {
                                    iced_native::Event::Keyboard(
                                        keyboard::Event::CharacterReceived(c),
                                    )
                                })
                                .collect();
                            events.append(&mut chars);
                        }
                    }
                    events
                }
                KeyboardEventVariant::Repeat(ke) => {
                    let mut skip_char = false;

                    let mut events: Vec<_> = keysym_to_vkey(ke.keysym)
                        .map(|k| {
                            if k == KeyCode::Backspace {
                                skip_char = true;
                            }
                            iced_native::Event::Keyboard(
                                keyboard::Event::KeyPressed {
                                    key_code: k,
                                    modifiers: modifiers_to_native(*modifiers),
                                },
                            )
                        })
                        .into_iter()
                        .collect();
                    if !skip_char {
                        if let Some(s) = ke.utf8 {
                            let mut chars = s
                                .chars()
                                .map(|c| {
                                    iced_native::Event::Keyboard(
                                        keyboard::Event::CharacterReceived(c),
                                    )
                                })
                                .collect();
                            events.append(&mut chars);
                        }
                    }
                    events
                }
                KeyboardEventVariant::Release(k) => keysym_to_vkey(k.keysym)
                    .map(|k| {
                        iced_native::Event::Keyboard(
                            keyboard::Event::KeyReleased {
                                key_code: k,
                                modifiers: modifiers_to_native(*modifiers),
                            },
                        )
                    })
                    .into_iter()
                    .collect(),
                KeyboardEventVariant::Modifiers(new_mods) => {
                    *modifiers = new_mods;
                    vec![iced_native::Event::Keyboard(
                        keyboard::Event::ModifiersChanged(modifiers_to_native(
                            new_mods,
                        )),
                    )]
                }
            },
            SctkEvent::WindowEvent {
                variant,
                id: surface,
            } => match variant {
                // TODO Ashley: platform specific events for window
                WindowEventVariant::Created(..) => Default::default(),
                WindowEventVariant::Close => destroyed_surface_ids
                    .get(&surface.id())
                    .map(|id| {
                        iced_native::Event::Window(
                            id.inner(),
                            window::Event::CloseRequested,
                        )
                    })
                    .into_iter()
                    .collect(),
                WindowEventVariant::WmCapabilities(_) => Default::default(),
                WindowEventVariant::ConfigureBounds { .. } => {
                    Default::default()
                }
                WindowEventVariant::Configure(configure, surface, _) => {
                    if configure.is_resizing() {
                        let new_size = configure.new_size.unwrap();
                        surface_ids
                            .get(&surface.id())
                            .map(|id| {
                                iced_native::Event::Window(
                                    id.inner(),
                                    window::Event::Resized {
                                        width: new_size.0,
                                        height: new_size.1,
                                    },
                                )
                            })
                            .into_iter()
                            .collect()
                    } else {
                        Default::default()
                    }
                }
            },
            SctkEvent::LayerSurfaceEvent {
                variant,
                id: surface,
            } => match variant {
                LayerSurfaceEventVariant::Done => destroyed_surface_ids
                    .get(&surface.id())
                    .map(|id| {
                        iced_native::Event::PlatformSpecific(
                            PlatformSpecific::Wayland(wayland::Event::Layer(
                                LayerEvent::Done,
                                surface,
                                id.inner(),
                            )),
                        )
                    })
                    .into_iter()
                    .collect(),
                _ => Default::default(),
            },
            SctkEvent::PopupEvent {
                variant,
                id: surface,
                ..
            } => {
                match variant {
                    PopupEventVariant::Done => destroyed_surface_ids
                        .get(&surface.id())
                        .map(|id| {
                            iced_native::Event::PlatformSpecific(
                                PlatformSpecific::Wayland(
                                    wayland::Event::Popup(
                                        PopupEvent::Done,
                                        surface,
                                        id.inner(),
                                    ),
                                ),
                            )
                        })
                        .into_iter()
                        .collect(),
                    PopupEventVariant::Created(_, _) => Default::default(), // TODO
                    PopupEventVariant::WmCapabilities(_) => Default::default(), // TODO
                    PopupEventVariant::Configure(_, _, _) => Default::default(), // TODO
                    PopupEventVariant::RepositionionedPopup { token } => {
                        Default::default()
                    } // TODO
                }
            }
            SctkEvent::NewOutput { id, info } => Default::default(),
            SctkEvent::UpdateOutput { id, info } => Default::default(),
            SctkEvent::RemovedOutput(_) => Default::default(),
            SctkEvent::Draw(_) => Default::default(),
            SctkEvent::ScaleFactorChanged {
                factor,
                id,
                inner_size,
            } => Default::default(),
        }
    }
}
