pub mod control_flow;
pub mod proxy;
pub mod state;

use std::{
    collections::HashMap,
    fmt::Debug,
    mem,
    time::{Duration, Instant},
};

use crate::{
    application::Event,
    sctk_event::{
        IcedSctkEvent, LayerSurfaceEventVariant, PopupEventVariant, SctkEvent,
        StartCause, SurfaceUserRequest, WindowEventVariant,
    },
    settings,
};

use iced_native::command::platform_specific::{
    self,
    wayland::{
        layer_surface::SctkLayerSurfaceSettings, window::SctkWindowSettings,
    },
};
use sctk::{
    compositor::CompositorState,
    event_loop::WaylandSource,
    output::OutputState,
    reexports::{
        calloop::{self, EventLoop},
        client::{
            backend::ObjectId, globals::registry_queue_init,
            protocol::wl_surface::WlSurface, ConnectError, Connection,
            DispatchError, Proxy,
        },
    },
    registry::RegistryState,
    seat::SeatState,
    shell::{
        layer::LayerShell,
        xdg::{window::XdgWindowState, XdgShellState},
    },
    shm::ShmState,
};
use wayland_backend::client::WaylandError;

use self::{
    control_flow::ControlFlow,
    state::{LayerSurfaceCreationError, SctkState},
};

// impl SctkSurface {
//     pub fn hash(&self) -> u64 {
//         let hasher = DefaultHasher::new();
//         match self {
//             SctkSurface::LayerSurface(s) => s.wl_surface().id().hash(.hash(&mut hasher)),
//             SctkSurface::Window(s) => s.wl_surface().id().hash(.hash(&mut hasher)),
//             SctkSurface::Popup(s) => s.wl_surface().id().hash(.hash(&mut hasher)),
//         };
//         hasher.finish()
//     }
// }

#[derive(Debug, Default, Clone, Copy)]
pub struct Features {
    // TODO
}

#[derive(Debug)]
pub struct SctkEventLoop<T> {
    // TODO after merged
    // pub data_device_manager_state: DataDeviceManagerState,
    pub(crate) event_loop: EventLoop<'static, SctkState<T>>,
    pub(crate) wayland_dispatcher:
        calloop::Dispatcher<'static, WaylandSource<SctkState<T>>, SctkState<T>>,
    pub(crate) features: Features,
    /// A proxy to wake up event loop.
    pub event_loop_awakener: calloop::ping::Ping,
    /// A sender for submitting user events in the event loop
    pub user_events_sender: calloop::channel::Sender<Event<T>>,
    pub(crate) state: SctkState<T>,
}

impl<T> SctkEventLoop<T>
where
    T: 'static + Debug,
{
    pub(crate) fn new<F: Sized>(
        _settings: &settings::Settings<F>,
    ) -> Result<Self, ConnectError> {
        let connection = Connection::connect_to_env()?;
        let _display = connection.display();
        let (globals, event_queue) = registry_queue_init(&connection).unwrap();
        let event_loop = calloop::EventLoop::<SctkState<T>>::try_new().unwrap();
        let loop_handle = event_loop.handle();

        let qh = event_queue.handle();
        let registry_state = RegistryState::new(&globals);

        let (ping, ping_source) = calloop::ping::make_ping().unwrap();
        // TODO
        loop_handle
            .insert_source(ping_source, |_, _, _state| {
                // Drain events here as well to account for application doing batch event processing
                // on RedrawEventsCleared.
                // shim::handle_window_requests(state);
                todo!()
            })
            .unwrap();
        let (user_events_sender, user_events_channel) =
            calloop::channel::channel();

        loop_handle
            .insert_source(user_events_channel, |event, _, state| match event {
                calloop::channel::Event::Msg(e) => {
                    state.pending_user_events.push(e);
                }
                calloop::channel::Event::Closed => {}
            })
            .unwrap();
        let wayland_source = WaylandSource::new(event_queue).unwrap();

        let wayland_dispatcher = calloop::Dispatcher::new(
            wayland_source,
            |_, queue, winit_state| queue.dispatch_pending(winit_state),
        );

        let _wayland_source_dispatcher = event_loop
            .handle()
            .register_dispatcher(wayland_dispatcher.clone())
            .unwrap();

        Ok(Self {
            event_loop,
            wayland_dispatcher,
            state: SctkState {
                connection,
                registry_state,
                seat_state: SeatState::new(&globals, &qh),
                output_state: OutputState::new(&globals, &qh),
                compositor_state: CompositorState::bind(&globals, &qh)
                    .expect("wl_compositor is not available"),
                shm_state: ShmState::bind(&globals, &qh)
                    .expect("wl_shm is not available"),
                xdg_shell_state: XdgShellState::bind(&globals, &qh)
                    .expect("xdg shell is not available"),
                xdg_window_state: XdgWindowState::bind(&globals, &qh),
                layer_shell: LayerShell::bind(&globals, &qh).ok(),

                // data_device_manager_state: DataDeviceManagerState::new(),
                queue_handle: qh,
                loop_handle: loop_handle,

                cursor_surface: None,
                multipool: None,
                outputs: Vec::new(),
                seats: Vec::new(),
                windows: Vec::new(),
                layer_surfaces: Vec::new(),
                popups: Vec::new(),
                kbd_focus: None,
                window_user_requests: HashMap::new(),
                window_compositor_updates: HashMap::new(),
                sctk_events: Vec::new(),
                popup_compositor_updates: Default::default(),
                layer_surface_compositor_updates: Default::default(),
                layer_surface_user_requests: Default::default(),
                popup_user_requests: Default::default(),
                pending_user_events: Vec::new(),
            },
            features: Default::default(),
            event_loop_awakener: ping,
            user_events_sender,
        })
    }

    pub fn proxy(&self) -> proxy::Proxy<Event<T>> {
        proxy::Proxy::new(self.user_events_sender.clone())
    }

    pub fn get_layer_surface(
        &mut self,
        layer_surface: SctkLayerSurfaceSettings,
    ) -> Result<(iced_native::window::Id, WlSurface), LayerSurfaceCreationError>
    {
        self.state.get_layer_surface(layer_surface)
    }

    pub fn get_window(
        &mut self,
        settings: SctkWindowSettings,
    ) -> (iced_native::window::Id, WlSurface) {
        self.state.get_window(settings)
    }

    pub fn run_return<F>(&mut self, mut callback: F) -> i32
    where
        F: FnMut(IcedSctkEvent<T>, &SctkState<T>, &mut ControlFlow),
    {
        let mut control_flow = ControlFlow::Poll;

        callback(
            IcedSctkEvent::NewEvents(StartCause::Init),
            &self.state,
            &mut control_flow,
        );

        let mut surface_user_requests: Vec<(ObjectId, SurfaceUserRequest)> =
            Vec::new();

        let mut event_sink_back_buffer = Vec::new();

        // NOTE We break on errors from dispatches, since if we've got protocol error
        // libwayland-client/wayland-rs will inform us anyway, but crashing downstream is not
        // really an option. Instead we inform that the event loop got destroyed. We may
        // communicate an error that something was terminated, but winit doesn't provide us
        // with an API to do that via some event.
        // Still, we set the exit code to the error's OS error code, or to 1 if not possible.
        let exit_code = loop {
            // Send pending events to the server.
            match self.state.connection.flush() {
                Ok(_) => {}
                Err(error) => {
                    break match error {
                        WaylandError::Io(err) => err.raw_os_error(),
                        WaylandError::Protocol(_) => None,
                    }
                    .unwrap_or(1)
                }
            }

            // During the run of the user callback, some other code monitoring and reading the
            // Wayland socket may have been run (mesa for example does this with vsync), if that
            // is the case, some events may have been enqueued in our event queue.
            //
            // If some messages are there, the event loop needs to behave as if it was instantly
            // woken up by messages arriving from the Wayland socket, to avoid delaying the
            // dispatch of these events until we're woken up again.
            let instant_wakeup = {
                let mut wayland_source =
                    self.wayland_dispatcher.as_source_mut();
                let queue = wayland_source.queue();
                match queue.dispatch_pending(&mut self.state) {
                    Ok(dispatched) => dispatched > 0,
                    // TODO better error handling
                    Err(error) => {
                        break match error {
                            DispatchError::BadMessage { .. } => None,
                            DispatchError::Backend(err) => match err {
                                WaylandError::Io(err) => err.raw_os_error(),
                                WaylandError::Protocol(_) => None,
                            },
                        }
                        .unwrap_or(1)
                    }
                }
            };

            match control_flow {
                ControlFlow::ExitWithCode(code) => break code,
                ControlFlow::Poll => {
                    // Non-blocking dispatch.
                    let timeout = Duration::from_millis(0);
                    if let Err(error) =
                        self.event_loop.dispatch(Some(timeout), &mut self.state)
                    {
                        break raw_os_err(error);
                    }

                    callback(
                        IcedSctkEvent::NewEvents(StartCause::Poll),
                        &self.state,
                        &mut control_flow,
                    );
                }
                ControlFlow::Wait => {
                    let timeout = if instant_wakeup {
                        Some(Duration::from_millis(0))
                    } else {
                        None
                    };

                    if let Err(error) =
                        self.event_loop.dispatch(timeout, &mut self.state)
                    {
                        break raw_os_err(error);
                    }

                    callback(
                        IcedSctkEvent::NewEvents(StartCause::WaitCancelled {
                            start: Instant::now(),
                            requested_resume: None,
                        }),
                        &self.state,
                        &mut control_flow,
                    );
                }
                ControlFlow::WaitUntil(deadline) => {
                    let start = Instant::now();

                    // Compute the amount of time we'll block for.
                    let duration = if deadline > start && !instant_wakeup {
                        deadline - start
                    } else {
                        Duration::from_millis(0)
                    };

                    if let Err(error) = self
                        .event_loop
                        .dispatch(Some(duration), &mut self.state)
                    {
                        break raw_os_err(error);
                    }

                    let now = Instant::now();

                    if now < deadline {
                        callback(
                            IcedSctkEvent::NewEvents(
                                StartCause::WaitCancelled {
                                    start,
                                    requested_resume: Some(deadline),
                                },
                            ),
                            &self.state,
                            &mut control_flow,
                        )
                    } else {
                        callback(
                            IcedSctkEvent::NewEvents(
                                StartCause::ResumeTimeReached {
                                    start,
                                    requested_resume: deadline,
                                },
                            ),
                            &self.state,
                            &mut control_flow,
                        )
                    }
                }
            }

            // The purpose of the back buffer and that swap is to not hold borrow_mut when
            // we're doing callback to the user, since we can double borrow if the user decides
            // to create a window in one of those callbacks.
            std::mem::swap(
                &mut event_sink_back_buffer,
                &mut self.state.sctk_events,
            );

            // Handle pending sctk events.
            let mut must_redraw = Vec::new();

            for event in event_sink_back_buffer.drain(..) {
                match event {
                    SctkEvent::Draw(id) => must_redraw.push(id),
                    SctkEvent::PopupEvent {
                        variant: PopupEventVariant::Done,
                        toplevel_id,
                        parent_id,
                        id,
                    } => {
                        match self
                            .state
                            .popups
                            .iter()
                            .position(|s| s.popup.wl_surface().id() == id.id())
                        {
                            Some(p) => {
                                let _p = self.state.popups.remove(p);
                                sticky_exit_callback(
                                    IcedSctkEvent::SctkEvent(
                                        SctkEvent::PopupEvent {
                                            variant: PopupEventVariant::Done,
                                            toplevel_id,
                                            parent_id,
                                            id,
                                        },
                                    ),
                                    &self.state,
                                    &mut control_flow,
                                    &mut callback,
                                );
                            }
                            None => continue,
                        };
                    }
                    SctkEvent::LayerSurfaceEvent {
                        variant: LayerSurfaceEventVariant::Done,
                        id,
                    } => {
                        if let Some(i) =
                            self.state.layer_surfaces.iter().position(|l| {
                                l.surface.wl_surface().id() == id.id()
                            })
                        {
                            let _l = self.state.layer_surfaces.remove(i);
                            sticky_exit_callback(
                                IcedSctkEvent::SctkEvent(
                                    SctkEvent::LayerSurfaceEvent {
                                        variant: LayerSurfaceEventVariant::Done,
                                        id,
                                    },
                                ),
                                &self.state,
                                &mut control_flow,
                                &mut callback,
                            );
                        }
                    }
                    SctkEvent::WindowEvent {
                        variant: WindowEventVariant::Close,
                        id,
                    } => {
                        if let Some(i) =
                            self.state.windows.iter().position(|l| {
                                l.window.wl_surface().id() == id.id()
                            })
                        {
                            let w = self.state.windows.remove(i);
                            w.window.xdg_toplevel().destroy();
                            sticky_exit_callback(
                                IcedSctkEvent::SctkEvent(
                                    SctkEvent::WindowEvent {
                                        variant: WindowEventVariant::Close,
                                        id,
                                    },
                                ),
                                &self.state,
                                &mut control_flow,
                                &mut callback,
                            );
                        }
                    }
                    _ => sticky_exit_callback(
                        IcedSctkEvent::SctkEvent(event),
                        &self.state,
                        &mut control_flow,
                        &mut callback,
                    ),
                }
            }

            // handle events indirectly via callback to the user.
            let (sctk_events, user_events): (Vec<_>, Vec<_>) = self
                .state
                .pending_user_events
                .drain(..)
                .partition(|e| matches!(e, Event::SctkEvent(_)));
            let mut to_commit = HashMap::new();
            for event in sctk_events.into_iter().chain(user_events.into_iter())
            {
                match event {
                    Event::SctkEvent(event) => {
                        sticky_exit_callback(event, &self.state, &mut control_flow, &mut callback)
                    }
                    Event::LayerSurface(action) => match action {
                        platform_specific::wayland::layer_surface::Action::LayerSurface {
                            builder,
                            _phantom,
                        } => {
                            // TODO ASHLEY: error handling
                            if let Ok((id, wl_surface)) = self.state.get_layer_surface(builder) {
                                let object_id = wl_surface.id();
                                sticky_exit_callback(
                                    IcedSctkEvent::SctkEvent(SctkEvent::LayerSurfaceEvent {
                                        variant: LayerSurfaceEventVariant::Created(object_id.clone(), id),
                                        id: wl_surface.clone(),
                                    }),
                                    &self.state,
                                    &mut control_flow,
                                    &mut callback,
                                );
                            }
                        }
                        platform_specific::wayland::layer_surface::Action::Size {
                            id,
                            width,
                            height,
                        } => {
                            if let Some(layer_surface) = self.state.layer_surfaces.iter_mut().find(|l| l.id == id) {
                                layer_surface.requested_size = (width, height);
                                layer_surface.surface.set_size(width.unwrap_or_default(), height.unwrap_or_default());
                                to_commit.insert(id, layer_surface.surface.wl_surface().clone());
                            }
                        },
                        platform_specific::wayland::layer_surface::Action::Destroy(id) => {
                            if let Some(i) = self.state.layer_surfaces.iter().position(|l| &l.id == &id) {
                                let l = self.state.layer_surfaces.remove(i);
                                sticky_exit_callback(
                                    IcedSctkEvent::SctkEvent(SctkEvent::LayerSurfaceEvent {
                                        variant: LayerSurfaceEventVariant::Done,
                                        id: l.surface.wl_surface().clone(),
                                    }),
                                    &self.state,
                                    &mut control_flow,
                                    &mut callback,
                                );
                            }
                        },
                        platform_specific::wayland::layer_surface::Action::Anchor { id, anchor } => {
                            if let Some(layer_surface) = self.state.layer_surfaces.iter_mut().find(|l| l.id == id) {
                                layer_surface.anchor = anchor;
                                layer_surface.surface.set_anchor(anchor);
                                to_commit.insert(id, layer_surface.surface.wl_surface().clone());

                            }
                        }
                        platform_specific::wayland::layer_surface::Action::ExclusiveZone {
                            id,
                            exclusive_zone,
                        } => {
                            if let Some(layer_surface) = self.state.layer_surfaces.iter_mut().find(|l| l.id == id) {
                                layer_surface.exclusive_zone = exclusive_zone;
                                layer_surface.surface.set_exclusive_zone(exclusive_zone);
                                to_commit.insert(id, layer_surface.surface.wl_surface().clone());
                            }
                        },
                        platform_specific::wayland::layer_surface::Action::Margin {
                            id,
                            margin,
                        } => {
                            if let Some(layer_surface) = self.state.layer_surfaces.iter_mut().find(|l| l.id == id) {
                                layer_surface.margin = margin;
                                layer_surface.surface.set_margin(margin.top, margin.right, margin.bottom, margin.left);
                                to_commit.insert(id, layer_surface.surface.wl_surface().clone());
                            }
                        },
                        platform_specific::wayland::layer_surface::Action::KeyboardInteractivity { id, keyboard_interactivity } => {
                            if let Some(layer_surface) = self.state.layer_surfaces.iter_mut().find(|l| l.id == id) {
                                layer_surface.keyboard_interactivity = keyboard_interactivity;
                                layer_surface.surface.set_keyboard_interactivity(keyboard_interactivity);
                                to_commit.insert(id, layer_surface.surface.wl_surface().clone());

                            }
                        },
                        platform_specific::wayland::layer_surface::Action::Layer { id, layer } => {
                            if let Some(layer_surface) = self.state.layer_surfaces.iter_mut().find(|l| l.id == id) {
                                layer_surface.layer = layer;
                                layer_surface.surface.set_layer(layer);
                                to_commit.insert(id, layer_surface.surface.wl_surface().clone());

                            }
                        },
                    },
                    Event::SetCursor(_) => {
                        // TODO set cursor after cursor theming PR is merged
                        // https://github.com/Smithay/client-toolkit/pull/306
                    }
                    Event::Window(action) => match action {
                        platform_specific::wayland::window::Action::Window { builder, _phantom } => {
                            let (id, wl_surface) = self.state.get_window(builder);
                            let object_id = wl_surface.id();
                            sticky_exit_callback(
                                IcedSctkEvent::SctkEvent(SctkEvent::WindowEvent { variant: WindowEventVariant::Created(object_id.clone(), id), id: wl_surface.clone() }),
                                &self.state,
                                &mut control_flow,
                                &mut callback,
                            );
                        },
                        platform_specific::wayland::window::Action::Size { id, width, height } => {
                            if let Some(window) = self.state.windows.iter_mut().find(|w| w.id == id) {
                                window.requested_size = Some((width, height));
                                window.window.xdg_surface().set_window_geometry(0, 0, width.max(1) as i32, height.max(1) as i32);
                                to_commit.insert(id, window.window.wl_surface().clone());
                                // TODO Ashley maybe don't force window size?
                                if let Some(mut prev_configure) = window.last_configure.clone() {
                                    prev_configure.new_size = Some((width, height));
                                    sticky_exit_callback(
                                        IcedSctkEvent::SctkEvent(SctkEvent::WindowEvent { variant: WindowEventVariant::Configure(prev_configure, window.window.wl_surface().clone(), false), id: window.window.wl_surface().clone()}),
                                        &self.state,
                                        &mut control_flow,
                                        &mut callback,
                                    );
                                }
                            }
                        },
                        platform_specific::wayland::window::Action::MinSize { id, size } => {
                            if let Some(window) = self.state.windows.iter_mut().find(|w| w.id == id) {
                                window.window.set_min_size(size);
                                to_commit.insert(id, window.window.wl_surface().clone());
                            }
                        },
                        platform_specific::wayland::window::Action::MaxSize { id, size } => {
                            if let Some(window) = self.state.windows.iter_mut().find(|w| w.id == id) {
                                window.window.set_max_size(size);
                                to_commit.insert(id, window.window.wl_surface().clone());
                            }
                        },
                        platform_specific::wayland::window::Action::Title { id, title } => {
                            if let Some(window) = self.state.windows.iter_mut().find(|w| w.id == id) {
                                window.window.set_title(title);
                                to_commit.insert(id, window.window.wl_surface().clone());
                            }
                        },
                        platform_specific::wayland::window::Action::Minimize { id } => {
                            if let Some(window) = self.state.windows.iter_mut().find(|w| w.id == id) {
                                window.window.set_mimimized();
                                to_commit.insert(id, window.window.wl_surface().clone());
                            }
                        },
                        platform_specific::wayland::window::Action::Maximize { id } => {
                            if let Some(window) = self.state.windows.iter_mut().find(|w| w.id == id) {
                                window.window.set_maximized();
                                to_commit.insert(id, window.window.wl_surface().clone());
                            }
                        },
                        platform_specific::wayland::window::Action::UnsetMaximize { id } => {
                            if let Some(window) = self.state.windows.iter_mut().find(|w| w.id == id) {
                                window.window.unset_maximized();
                                to_commit.insert(id, window.window.wl_surface().clone());
                            }
                        },
                        platform_specific::wayland::window::Action::Fullscreen { id } => {
                            if let Some(window) = self.state.windows.iter_mut().find(|w| w.id == id) {
                                // TODO ASHLEY: allow specific output to be requested for fullscreen?
                                window.window.set_fullscreen(None);
                                to_commit.insert(id, window.window.wl_surface().clone());
                            }
                        },
                        platform_specific::wayland::window::Action::UnsetFullscreen { id } => {
                            if let Some(window) = self.state.windows.iter_mut().find(|w| w.id == id) {
                                window.window.unset_fullscreen();
                                to_commit.insert(id, window.window.wl_surface().clone());
                            }
                        },
                        platform_specific::wayland::window::Action::InteractiveMove { id } => {
                            if let (Some(window), Some((seat, last_press))) = (self.state.windows.iter_mut().find(|w| w.id == id), self.state.seats.first().and_then(|seat| seat.last_ptr_press.map(|p| (&seat.seat, p.2)))) {
                                window.window.xdg_toplevel()._move(seat, last_press);
                                to_commit.insert(id, window.window.wl_surface().clone());
                            }
                        },
                        platform_specific::wayland::window::Action::InteractiveResize { id, edge } => {
                            if let (Some(window), Some((seat, last_press))) = (self.state.windows.iter_mut().find(|w| w.id == id), self.state.seats.first().and_then(|seat| seat.last_ptr_press.map(|p| (&seat.seat, p.2)))) {
                                window.window.xdg_toplevel().resize(seat, last_press, edge);
                                to_commit.insert(id, window.window.wl_surface().clone());
                            }
                        },
                        platform_specific::wayland::window::Action::ToggleMaximized { id } => {
                            if let Some(window) = self.state.windows.iter_mut().find(|w| w.id == id) {
                                if let Some(c) = &window.last_configure {
                                    dbg!(c);
                                    if c.is_maximized() {
                                        window.window.unset_maximized();
                                    } else {
                                        window.window.set_maximized();
                                    }
                                    to_commit.insert(id, window.window.wl_surface().clone());
                                }
                            }
                        },
                        platform_specific::wayland::window::Action::ShowWindowMenu { id, x, y } => todo!(),
                        platform_specific::wayland::window::Action::Destroy(id) => {
                            if let Some(i) = self.state.windows.iter().position(|l| &l.id == &id) {
                                let window = self.state.windows.remove(i);
                                window.window.xdg_toplevel().destroy();
                                sticky_exit_callback(
                                    IcedSctkEvent::SctkEvent(SctkEvent::WindowEvent {
                                        variant: WindowEventVariant::Close,
                                        id: window.window.wl_surface().clone(),
                                    }),
                                    &self.state,
                                    &mut control_flow,
                                    &mut callback,
                                );
                            }
                        },
                        platform_specific::wayland::window::Action::Mode(id, mode) => {
                            if let Some(window) = self.state.windows.iter_mut().find(|w| w.id == id) {
                                match mode {
                                    iced_native::window::Mode::Windowed => {
                                        window.window.unset_fullscreen();
                                    },
                                    iced_native::window::Mode::Fullscreen => {
                                        window.window.set_fullscreen(None);
                                    },
                                    iced_native::window::Mode::Hidden => {
                                        window.window.set_mimimized();
                                    },
                                }
                                to_commit.insert(id, window.window.wl_surface().clone());
                            }
                        },
                        platform_specific::wayland::window::Action::ToggleFullscreen { id } => {
                            if let Some(window) = self.state.windows.iter_mut().find(|w| w.id == id) {
                                if let Some(c) = &window.last_configure {
                                    if c.is_fullscreen() {
                                        window.window.unset_fullscreen();
                                    } else {
                                        window.window.set_fullscreen(None);
                                    }
                                    to_commit.insert(id, window.window.wl_surface().clone());
                                }
                            }
                        },
                        platform_specific::wayland::window::Action::AppId { id, app_id } => {
                            if let Some(window) = self.state.windows.iter_mut().find(|w| w.id == id) {
                                window.window.set_app_id(app_id);
                                to_commit.insert(id, window.window.wl_surface().clone());
                            }
                        },
                    },
                    Event::Popup(action) => match action {
                        platform_specific::wayland::popup::Action::Popup { popup, .. } => {
                            if let Ok((id, parent_id, toplevel_id, wl_surface)) = self.state.get_popup(popup) {
                                let object_id = wl_surface.id();
                                sticky_exit_callback(
                                    IcedSctkEvent::SctkEvent(SctkEvent::PopupEvent { variant: crate::sctk_event::PopupEventVariant::Created(object_id.clone(), id), toplevel_id, parent_id, id: wl_surface.clone() }),
                                    &self.state,
                                    &mut control_flow,
                                    &mut callback,
                                );
                            }
                        },
                        // XXX popup destruction must be done carefully
                        // first destroy the uppermost popup, then work down to the requested popup
                        platform_specific::wayland::popup::Action::Destroy { id } => {
                            let sctk_popup = match self.state
                                .popups
                                .iter()
                                .position(|s| s.id == id)
                            {
                                Some(p) => self.state.popups.remove(p),
                                None => continue,
                            };
                            let mut to_destroy = vec![sctk_popup];
                            while let Some(popup_to_destroy) = to_destroy.last() {
                                match popup_to_destroy.parent.clone() {
                                    state::SctkSurface::LayerSurface(_) | state::SctkSurface::Window(_) => {
                                        break;
                                    }
                                    state::SctkSurface::Popup(popup_to_destroy_first) => {
                                        let popup_to_destroy_first = self
                                            .state
                                            .popups
                                            .iter()
                                            .position(|p| p.popup.wl_surface() == &popup_to_destroy_first)
                                            .unwrap();
                                        let popup_to_destroy_first = self.state.popups.remove(popup_to_destroy_first);
                                        to_destroy.push(popup_to_destroy_first);
                                    }
                                }
                            }
                            for popup in to_destroy.into_iter().rev() {
                                sticky_exit_callback(IcedSctkEvent::SctkEvent(SctkEvent::PopupEvent {
                                    variant: PopupEventVariant::Done,
                                    toplevel_id: popup.toplevel.clone(),
                                    parent_id: popup.parent.wl_surface().clone(),
                                    id: popup.popup.wl_surface().clone(),
                                }),
                                    &self.state,
                                    &mut control_flow,
                                    &mut callback,
                                );
                            }
                        },
                        platform_specific::wayland::popup::Action::Reposition { id, positioner } => todo!(),
                        platform_specific::wayland::popup::Action::Grab { id } => todo!(),
                    },
                }
            }

            // commit changes made via actions
            for s in to_commit {
                s.1.commit();
            }

            // Send events cleared.
            sticky_exit_callback(
                IcedSctkEvent::MainEventsCleared,
                &self.state,
                &mut control_flow,
                &mut callback,
            );

            // Apply user requests, so every event required resize and latter surface commit will
            // be applied right before drawing. This will also ensure that every `RedrawRequested`
            // event will be delivered in time.
            // Process 'new' pending updates from compositor.
            surface_user_requests.clear();
            surface_user_requests.extend(
                self.state.window_user_requests.iter_mut().map(
                    |(wid, window_request)| {
                        (wid.clone(), mem::take(window_request))
                    },
                ),
            );

            // Handle RedrawRequested requests.
            for (surface_id, mut surface_request) in
                surface_user_requests.iter()
            {
                if let Some(i) =
                    must_redraw.iter().position(|a_id| &a_id.id() == surface_id)
                {
                    must_redraw.remove(i);
                }
                let wl_suface = self
                    .state
                    .windows
                    .iter()
                    .map(|w| w.window.wl_surface())
                    .chain(
                        self.state
                            .layer_surfaces
                            .iter()
                            .map(|l| l.surface.wl_surface()),
                    )
                    .find(|s| s.id() == *surface_id)
                    .unwrap();

                // Handle refresh of the frame.
                if surface_request.refresh_frame {
                    // In general refreshing the frame requires surface commit, those force user
                    // to redraw.
                    surface_request.redraw_requested = true;
                }

                // Handle redraw request.
                if surface_request.redraw_requested {
                    sticky_exit_callback(
                        IcedSctkEvent::RedrawRequested(surface_id.clone()),
                        &self.state,
                        &mut control_flow,
                        &mut callback,
                    );
                }
                wl_suface.commit();
            }

            for id in must_redraw {
                sticky_exit_callback(
                    IcedSctkEvent::RedrawRequested(id.id()),
                    &self.state,
                    &mut control_flow,
                    &mut callback,
                );
            }

            // Send RedrawEventCleared.
            sticky_exit_callback(
                IcedSctkEvent::RedrawEventsCleared,
                &self.state,
                &mut control_flow,
                &mut callback,
            );
        };

        callback(IcedSctkEvent::LoopDestroyed, &self.state, &mut control_flow);
        exit_code
    }
}

fn sticky_exit_callback<T, F>(
    evt: IcedSctkEvent<T>,
    target: &SctkState<T>,
    control_flow: &mut ControlFlow,
    callback: &mut F,
) where
    F: FnMut(IcedSctkEvent<T>, &SctkState<T>, &mut ControlFlow),
{
    // make ControlFlow::ExitWithCode sticky by providing a dummy
    // control flow reference if it is already ExitWithCode.
    if let ControlFlow::ExitWithCode(code) = *control_flow {
        callback(evt, target, &mut ControlFlow::ExitWithCode(code))
    } else {
        callback(evt, target, control_flow)
    }
}

fn raw_os_err(err: calloop::Error) -> i32 {
    match err {
        calloop::Error::IoError(err) => err.raw_os_error(),
        _ => None,
    }
    .unwrap_or(1)
}
