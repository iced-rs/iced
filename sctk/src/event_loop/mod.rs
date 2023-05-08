pub mod control_flow;
pub mod proxy;
pub mod state;
#[cfg(feature = "a11y")]
pub mod adapter;


use crate::{
    application::{Event, SurfaceIdWrapper},
    sctk_event::{
        DndOfferEvent, IcedSctkEvent,
        LayerSurfaceEventVariant, PopupEventVariant, SctkEvent,
        SelectionOfferEvent, StartCause, WindowEventVariant,
    },
    settings,
};
use iced_futures::core::window::Mode;
use iced_runtime::command::platform_specific::{
    self,
    wayland::{
        data_device::DndIcon, layer_surface::SctkLayerSurfaceSettings,
        window::SctkWindowSettings,
    },
};
use log::error;
use sctk::data_device_manager::data_source::DragSource;
use sctk::{
    compositor::CompositorState,
    data_device_manager::DataDeviceManagerState,
    output::OutputState,
    reexports::{
        calloop::{self, EventLoop},
        client::{
            globals::registry_queue_init, protocol::wl_surface::WlSurface,
            ConnectError, Connection, DispatchError, Proxy, WaylandSource,
        },
    },
    registry::RegistryState,
    seat::SeatState,
    shell::{
        wlr_layer::LayerShell,
        xdg::{XdgShell, XdgSurface},
        WaylandSurface,
    },
    shm::Shm,
};
use std::{
    collections::HashMap,
    fmt::Debug,
    io::{BufRead, BufReader, BufWriter, Write},
    num::NonZeroU32,
    time::{Duration, Instant},
    sync::{Arc, Mutex}

};
use wayland_backend::client::WaylandError;

use self::{
    control_flow::ControlFlow,
    state::{Dnd, LayerSurfaceCreationError, SctkCopyPasteSource, SctkState},
};

#[derive(Debug, Default, Clone, Copy)]
pub struct Features {
    // TODO
}

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

    #[cfg(feature = "a11y")]
    pub(crate) a11y_events:
        Arc<Mutex<Vec<adapter::A11yWrapper>>>,
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
                shm_state: Shm::bind(&globals, &qh)
                    .expect("wl_shm is not available"),
                xdg_shell_state: XdgShell::bind(&globals, &qh)
                    .expect("xdg shell is not available"),
                layer_shell: LayerShell::bind(&globals, &qh).ok(),
                data_device_manager_state: DataDeviceManagerState::bind(
                    &globals, &qh,
                )
                .expect("data device manager is not available"),

                queue_handle: qh,
                loop_handle,

                cursor_surface: None,
                multipool: None,
                outputs: Vec::new(),
                seats: Vec::new(),
                windows: Vec::new(),
                layer_surfaces: Vec::new(),
                popups: Vec::new(),
                dnd_source: None,
                kbd_focus: None,
                window_compositor_updates: HashMap::new(),
                sctk_events: Vec::new(),
                popup_compositor_updates: Default::default(),
                layer_surface_compositor_updates: Default::default(),
                pending_user_events: Vec::new(),
                token_ctr: 0,
                selection_source: None,
                accept_counter: 0,
                dnd_offer: None,
                selection_offer: None,
            },
            features: Default::default(),
            event_loop_awakener: ping,
            user_events_sender,
            #[cfg(feature = "a11y")]
            a11y_events: Arc::new(Mutex::new(Vec::new())),
        })
    }

    pub fn proxy(&self) -> proxy::Proxy<Event<T>> {
        proxy::Proxy::new(self.user_events_sender.clone())
    }

    pub fn get_layer_surface(
        &mut self,
        layer_surface: SctkLayerSurfaceSettings,
    ) -> Result<(iced_runtime::window::Id, WlSurface), LayerSurfaceCreationError>
    {
        let ret = self.state.get_layer_surface(layer_surface);
        ret
    }

    pub fn get_window(
        &mut self,
        settings: SctkWindowSettings,
    ) -> (iced_runtime::window::Id, WlSurface) {
        let ret = self.state.get_window(settings);

        ret
    }

    // TODO Ashley provide users a reasonable method of setting the role for the surface
    #[cfg(feature = "a11y")]
    pub fn init_a11y_adapter(&mut self, surface: &WlSurface, app_id: Option<String>, surface_title: Option<String>, role: iced_accessibility::accesskit::Role) -> adapter::IcedSctkAdapter {
        use iced_accessibility::{accesskit_unix::Adapter, accesskit::{Node, Tree, TreeUpdate, Role, NodeId, NodeBuilder, NodeClassSet}, window_node_id};
        let node_id = window_node_id();
        // let node_id_clone = node_id.clone();
        let event_list = self.a11y_events.clone();
        adapter::IcedSctkAdapter {
            adapter: Adapter::new(app_id.unwrap_or_else(|| String::from("None")), "Iced".to_string(), env!("CARGO_PKG_VERSION").to_string(), move || {
                event_list.lock().unwrap().push(adapter::A11yWrapper::Enabled);
                let mut node = NodeBuilder::new(Role::Window);
                if let Some(name) = surface_title {
                    node.set_name(name);
                }
                let node = node.build(&mut NodeClassSet::lock_global());
                TreeUpdate {
                    nodes: vec![(
                        NodeId(node_id),
                        node,
                    )],
                    tree: Some(Tree::new(NodeId(node_id))),
                    focus: None,
                }
            }, Box::new(adapter::IcedSctkActionHandler {
                wl_surface: surface.clone(),
                event_list: self.a11y_events.clone(),
            })).unwrap(),
            id: node_id
        }
        
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

        let mut sctk_event_sink_back_buffer = Vec::new();

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
                &mut sctk_event_sink_back_buffer,
                &mut self.state.sctk_events,
            );
            
            // handle a11y events
            #[cfg(feature = "a11y")]
            if let Ok(mut events) = self.a11y_events.lock() {
                for event in events.drain(..) {
                    match event {
                        adapter::A11yWrapper::Enabled => sticky_exit_callback(
                            IcedSctkEvent::A11yEnabled,
                            &self.state,
                            &mut control_flow,
                            &mut callback,
                        ),
                        adapter::A11yWrapper::Event(event) => sticky_exit_callback(
                            IcedSctkEvent::A11yEvent(event),
                            &self.state,
                            &mut control_flow,
                            &mut callback,
                        ),
                    }
                }
            }
            // Handle pending sctk events.
            for event in sctk_event_sink_back_buffer.drain(..) {
                match event {
                    SctkEvent::Frame(id) => sticky_exit_callback(
                        IcedSctkEvent::SctkEvent(SctkEvent::Frame(id)),
                        &self.state,
                        &mut control_flow,
                        &mut callback,
                    ),
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
            let mut pending_redraws = Vec::new();
            for event in sctk_events.into_iter().chain(user_events.into_iter())
            {
                match event {
                    Event::Message(m) => {
                        sticky_exit_callback(
                            IcedSctkEvent::UserEvent(m),
                            &self.state,
                            &mut control_flow,
                            &mut callback,
                        );
                    }
                    Event::SctkEvent(event) => {
                        match event {
                            IcedSctkEvent::RedrawRequested(id) => {
                                pending_redraws.push(id);
                            },
                            e => sticky_exit_callback(
                                e,
                                &self.state,
                                &mut control_flow,
                                &mut callback,
                            ),
                        }
                    }
                    Event::LayerSurface(action) => match action {
                        platform_specific::wayland::layer_surface::Action::LayerSurface {
                            builder,
                            _phantom,
                        } => {
                            // TODO ASHLEY: error handling
                            if let Ok((id, wl_surface)) = self.state.get_layer_surface(builder) {
                                let object_id = wl_surface.id();
                                // TODO Ashley: all surfaces should probably have an optional title for a11y if nothing else
                                sticky_exit_callback(
                                    IcedSctkEvent::SctkEvent(SctkEvent::LayerSurfaceEvent {
                                        variant: LayerSurfaceEventVariant::Created(object_id.clone(), id),
                                        id: wl_surface.clone(),
                                    }),
                                    &self.state,
                                    &mut control_flow,
                                    &mut callback,
                                );
                                #[cfg(feature = "a11y")]
                                {
                                    let adapter = self.init_a11y_adapter(&wl_surface, None, None, iced_accessibility::accesskit::Role::Window);

                                    sticky_exit_callback(
                                        IcedSctkEvent::A11ySurfaceCreated(SurfaceIdWrapper::LayerSurface(id), adapter),
                                        &self.state,
                                        &mut control_flow,
                                        &mut callback,
                                    );
                                }
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

                                pending_redraws.push(layer_surface.surface.wl_surface().id());
                            }
                        },
                        platform_specific::wayland::layer_surface::Action::Destroy(id) => {
                            if let Some(i) = self.state.layer_surfaces.iter().position(|l| l.id == id) {
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
                            let app_id = builder.app_id.clone();
                            let title = builder.title.clone();
                            let (id, wl_surface) = self.state.get_window(builder);
                            let object_id = wl_surface.id();
                            sticky_exit_callback(
                                IcedSctkEvent::SctkEvent(SctkEvent::WindowEvent { 
                                    variant: WindowEventVariant::Created(object_id.clone(), id),
                                    id: wl_surface.clone() }),
                                &self.state,
                                &mut control_flow,
                                &mut callback,
                            );

                            #[cfg(feature = "a11y")]
                            {
                                let adapter = self.init_a11y_adapter(&wl_surface, app_id, title, iced_accessibility::accesskit::Role::Window);

                                sticky_exit_callback(
                                    IcedSctkEvent::A11ySurfaceCreated(SurfaceIdWrapper::LayerSurface(id), adapter),
                                    &self.state,
                                    &mut control_flow,
                                    &mut callback,
                                );
                            }
                        },
                        platform_specific::wayland::window::Action::Size { id, width, height } => {
                            if let Some(window) = self.state.windows.iter_mut().find(|w| w.id == id) {
                                let (width, height) = (NonZeroU32::new(width).unwrap_or(NonZeroU32::new(1).unwrap()), NonZeroU32::new(height).unwrap_or(NonZeroU32::new(1).unwrap()));
                                window.requested_size = Some((width.get(), height.get()));
                                window.window.xdg_surface().set_window_geometry(0, 0, width.get() as i32, height.get() as i32);
                                // TODO Ashley maybe don't force window size?
                                pending_redraws.push(window.window.wl_surface().id());

                                if let Some(mut prev_configure) = window.last_configure.clone() {
                                    prev_configure.new_size = (Some(width), Some(height));
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
                                window.window.set_minimized();
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
                            if let Some(i) = self.state.windows.iter().position(|l| l.id == id) {
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
                                    Mode::Windowed => {
                                        window.window.unset_fullscreen();
                                    },
                                    Mode::Fullscreen => {
                                        window.window.set_fullscreen(None);
                                    },
                                    Mode::Hidden => {
                                        window.window.set_minimized();
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
                                    IcedSctkEvent::SctkEvent(SctkEvent::PopupEvent {
                                        variant: crate::sctk_event::PopupEventVariant::Created(object_id.clone(), id),
                                        toplevel_id, parent_id, id: wl_surface.clone() }),
                                    &self.state,
                                    &mut control_flow,
                                    &mut callback,
                                );

                                #[cfg(feature = "a11y")]
                                {
                                let adapter = self.init_a11y_adapter(&wl_surface, None, None, iced_accessibility::accesskit::Role::Window);

                                sticky_exit_callback(
                                    IcedSctkEvent::A11ySurfaceCreated(SurfaceIdWrapper::LayerSurface(id), adapter),
                                    &self.state,
                                    &mut control_flow,
                                    &mut callback,
                                );
                            }
                            }
                        },
                        // XXX popup destruction must be done carefully
                        // first destroy the uppermost popup, then work down to the requested popup
                        platform_specific::wayland::popup::Action::Destroy { id } => {
                            let sctk_popup = match self.state
                                .popups
                                .iter()
                                .position(|s| s.data.id == id)
                            {
                                Some(p) => self.state.popups.remove(p),
                                None => continue,
                            };
                            let mut to_destroy = vec![sctk_popup];
                            while let Some(popup_to_destroy) = to_destroy.last() {
                                match popup_to_destroy.data.parent.clone() {
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
                                    toplevel_id: popup.data.toplevel.clone(),
                                    parent_id: popup.data.parent.wl_surface().clone(),
                                    id: popup.popup.wl_surface().clone(),
                                }),
                                    &self.state,
                                    &mut control_flow,
                                    &mut callback,
                                );
                            }
                        },
                        platform_specific::wayland::popup::Action::Size { id, width, height } => {
                            if let Some(sctk_popup) = self.state
                                .popups
                                .iter()
                                .find(|s| s.data.id == id)
                            {
                                // update geometry
                                sctk_popup.popup.xdg_surface().set_window_geometry(0, 0, width as i32, height as i32);
                                // update positioner
                                self.state.token_ctr += 1;
                                sctk_popup.data.positioner.set_size(width as i32, height as i32);
                                sctk_popup.popup.reposition(&sctk_popup.data.positioner, self.state.token_ctr);
                                pending_redraws.push(sctk_popup.popup.wl_surface().id());

                                sticky_exit_callback(IcedSctkEvent::SctkEvent(SctkEvent::PopupEvent {
                                    variant: PopupEventVariant::Size(width, height),
                                    toplevel_id: sctk_popup.data.toplevel.clone(),
                                    parent_id: sctk_popup.data.parent.wl_surface().clone(),
                                    id: sctk_popup.popup.wl_surface().clone(),
                                }),
                                    &self.state,
                                    &mut control_flow,
                                    &mut callback,
                                );
                            }
                        },
                        // TODO probably remove this?
                        platform_specific::wayland::popup::Action::Grab { .. } => {},
                    },
                    Event::DataDevice(action) => {
                        match action.inner {
                            platform_specific::wayland::data_device::ActionInner::Accept(mime_type) => {
                                let drag_offer = match self.state.dnd_offer.as_mut() {
                                    Some(d) => d,
                                    None => continue,
                                };
                                drag_offer.offer.accept_mime_type(drag_offer.offer.serial, mime_type);
                            }
                            platform_specific::wayland::data_device::ActionInner::StartInternalDnd { origin_id, icon_id } => {
                                let qh = &self.state.queue_handle.clone();
                                let seat = match self.state.seats.get(0) {
                                    Some(s) => s,
                                    None => continue,
                                };
                                let serial = match seat.last_ptr_press {
                                    Some(s) => s.2,
                                    None => continue,
                                };

                                let origin = match self
                                .state
                                .windows
                                .iter()
                                .find(|w| w.id == origin_id)
                                .map(|w| Some(w.window.wl_surface()))
                                .unwrap_or_else(|| self.state.layer_surfaces.iter()
                                                .find(|l| l.id == origin_id).map(|l| Some(l.surface.wl_surface()))
                                .unwrap_or_else(|| self.state.popups.iter().find(|p| p.data.id == origin_id).map(|p| p.popup.wl_surface()))) {
                                    Some(s) => s.clone(),
                                    None => continue,
                                };
                                let device = match self.state.seats.get(0) {
                                    Some(s) => &s.data_device,
                                    None => continue,
                                };
                                let icon_surface =  if let Some(icon_id) = icon_id{
                                    let wl_surface = self.state.compositor_state.create_surface(qh);
                                    DragSource::start_internal_drag(device, &origin, Some(&wl_surface), serial);
                                    Some((wl_surface, icon_id))
                                } else {
                                    DragSource::start_internal_drag(device, &origin, None, serial);
                                    None
                                };
                                self.state.dnd_source = Some(Dnd {
                                    origin_id,
                                    icon_surface,
                                    origin,
                                    source: None,
                                    pending_requests: Vec::new(),
                                    pipe: None,
                                    cur_write: None,
                                });
                            }
                            platform_specific::wayland::data_device::ActionInner::StartDnd { mime_types, actions, origin_id, icon_id, data } => {
                                if let Some(dnd_source) = self.state.dnd_source.as_ref() {
                                    if dnd_source.cur_write.is_some() {
                                        continue;
                                    }
                                }
                                let qh = &self.state.queue_handle.clone();
                                let seat = match self.state.seats.get(0) {
                                    Some(s) => s,
                                    None => continue,
                                };
                                let serial = match seat.last_ptr_press {
                                    Some(s) => s.2,
                                    None => continue,
                                };

                                let origin = match self
                                .state
                                .windows
                                .iter()
                                .find(|w| w.id == origin_id)
                                .map(|w| Some(w.window.wl_surface()))
                                .unwrap_or_else(|| self.state.layer_surfaces.iter()
                                                .find(|l| l.id == origin_id).map(|l| Some(l.surface.wl_surface()))
                                .unwrap_or_else(|| self.state.popups.iter().find(|p| p.data.id == origin_id).map(|p| p.popup.wl_surface()))) {
                                    Some(s) => s.clone(),
                                    None => continue,
                                };
                                let device = match self.state.seats.get(0) {
                                    Some(s) => &s.data_device,
                                    None => continue,
                                };
                                let source = self.state.data_device_manager_state.create_drag_and_drop_source(qh, mime_types.iter().map(|s| s.as_str()).collect::<Vec<_>>(), actions);
                                let icon_surface =  if let Some(icon_id) = icon_id{
                                    let icon_native_id = match &icon_id {
                                        DndIcon::Custom(icon_id) => icon_id.clone(),
                                        DndIcon::Widget(icon_id, _) => icon_id.clone(),
                                    };
                                    let wl_surface = self.state.compositor_state.create_surface(qh);
                                    source.start_drag(device, &origin, Some(&wl_surface), serial);
                                    sticky_exit_callback(
                                        IcedSctkEvent::DndSurfaceCreated(
                                                    wl_surface.clone(),
                                                    icon_id,
                                                    origin_id)
                                                ,
                                            &self.state,
                                            &mut control_flow,
                                            &mut callback
                                    );
                                   Some((wl_surface, icon_native_id))
                                } else {
                                    source.start_drag(device, &origin, None, serial);
                                    None
                                };
                                self.state.dnd_source = Some(Dnd { origin_id, origin, source: Some((source, data)), icon_surface, pending_requests: Vec::new(), pipe: None, cur_write: None });
                            },
                            platform_specific::wayland::data_device::ActionInner::DndFinished => {
                                if let Some(offer) = self.state.dnd_offer.take() {
                                    if offer.dropped {
                                        offer.offer.finish();
                                    }
                                    else {
                                        self.state.dnd_offer = Some(offer);
                                    }
                               }
                            },
                            platform_specific::wayland::data_device::ActionInner::DndCancelled => {
                                if let Some(source) = self.state.dnd_source.as_mut() {
                                    source.source = None;
                                }
                            },
                            platform_specific::wayland::data_device::ActionInner::RequestDndData (mime_type) => {
                                if let Some(dnd_offer) = self.state.dnd_offer.as_mut() {
                                    let read_pipe = match dnd_offer.offer.receive(mime_type.clone()) {
                                        Ok(p) => p,
                                        Err(_) => continue, // TODO error handling
                                    };
                                    let loop_handle = self.event_loop.handle();
                                    match self.event_loop.handle().insert_source(read_pipe, move |_, f, state| {
                                        let mut dnd_offer = match state.dnd_offer.take() {
                                            Some(s) => s,
                                            None => return,
                                        };
                                        let (mime_type, data, token) = match dnd_offer.cur_read.take() {
                                            Some(s) => s,
                                            None => return,
                                        };
                                        let mut reader = BufReader::new(f);
                                        let consumed = match reader.fill_buf() {
                                            Ok(buf) => {
                                                if buf.is_empty() {
                                                    loop_handle.remove(token);
                                                    state.sctk_events.push(SctkEvent::DndOffer { event: DndOfferEvent::Data { data, mime_type }, surface: dnd_offer.offer.surface.clone() });
                                                    if dnd_offer.dropped {
                                                        dnd_offer.offer.finish();
                                                    } else {
                                                        state.dnd_offer = Some(dnd_offer);
                                                    }
                                                } else {
                                                    let mut data = data;
                                                    data.extend_from_slice(buf);
                                                    dnd_offer.cur_read = Some((mime_type, data, token));
                                                    state.dnd_offer = Some(dnd_offer);
                                                }
                                                buf.len()
                                            },
                                            Err(e) if matches!(e.kind(), std::io::ErrorKind::Interrupted) => {
                                                dnd_offer.cur_read = Some((mime_type, data, token));
                                                state.dnd_offer = Some(dnd_offer);
                                                return;
                                            },
                                            Err(e) => {
                                                error!("Error reading selection data: {}", e);
                                                loop_handle.remove(token);
                                                if !dnd_offer.dropped {
                                                    state.dnd_offer = Some(dnd_offer);
                                                }
                                                return;
                                            },
                                        };
                                        reader.consume(consumed);
                                    }) {
                                        Ok(token) => {
                                            dnd_offer.cur_read = Some((mime_type.clone(), Vec::new(), token));
                                        },
                                        Err(_) => continue,
                                    };
                                }
                            }
                            platform_specific::wayland::data_device::ActionInner::RequestSelectionData { mime_type } => {
                                if let Some(selection_offer) = self.state.selection_offer.as_mut() {
                                    let read_pipe = match selection_offer.offer.receive(mime_type.clone()) {
                                        Ok(p) => p,
                                        Err(_) => continue, // TODO error handling
                                    };
                                    let loop_handle = self.event_loop.handle();
                                    match self.event_loop.handle().insert_source(read_pipe, move |_, f, state| {
                                        let selection_offer = match state.selection_offer.as_mut() {
                                            Some(s) => s,
                                            None => return,
                                        };
                                        let (mime_type, data, token) = match selection_offer.cur_read.take() {
                                            Some(s) => s,
                                            None => return,
                                        };
                                        let mut reader = BufReader::new(f);
                                        let consumed = match reader.fill_buf() {
                                            Ok(buf) => {
                                                if buf.is_empty() {
                                                    loop_handle.remove(token);
                                                    state.sctk_events.push(SctkEvent::SelectionOffer(SelectionOfferEvent::Data {mime_type, data }));
                                                } else {
                                                    let mut data = data;
                                                    data.extend_from_slice(buf);
                                                    selection_offer.cur_read = Some((mime_type, data, token));
                                                }
                                                buf.len()
                                            },
                                            Err(e) if matches!(e.kind(), std::io::ErrorKind::Interrupted) => {
                                                selection_offer.cur_read = Some((mime_type, data, token));
                                                return;
                                            },
                                            Err(e) => {
                                                error!("Error reading selection data: {}", e);
                                                loop_handle.remove(token);
                                                return;
                                            },
                                        };
                                        reader.consume(consumed);
                                    }) {
                                        Ok(token) => {
                                            selection_offer.cur_read = Some((mime_type.clone(), Vec::new(), token));
                                        },
                                        Err(_) => continue,
                                    };
                                }
                            }
                            platform_specific::wayland::data_device::ActionInner::SetSelection { mime_types, data } => {
                                let qh = &self.state.queue_handle.clone();
                                let seat = match self.state.seats.get(0) {
                                    Some(s) => s,
                                    None => continue,
                                };
                                let serial = match seat.last_ptr_press {
                                    Some(s) => s.2,
                                    None => continue,
                                };
                                // remove the old selection
                                self.state.selection_source = None;
                                // create a new one
                                let source = self
                                    .state
                                    .data_device_manager_state
                                    .create_copy_paste_source(&qh, mime_types.iter().map(|s| s.as_str()).collect::<Vec<_>>());
                                source.set_selection(&seat.data_device, serial);
                                self.state.selection_source = Some(SctkCopyPasteSource {
                                    source,
                                    cur_write: None,
                                    accepted_mime_types: Vec::new(),
                                    pipe: None,
                                    data,
                                });
                            }
                            platform_specific::wayland::data_device::ActionInner::UnsetSelection => {
                                let seat = match self.state.seats.get(0) {
                                    Some(s) => s,
                                    None => continue,
                                };
                                let serial = match seat.last_ptr_press {
                                    Some(s) => s.2,
                                    None => continue,
                                };
                                self.state.selection_source = None;
                                seat.data_device.unset_selection(serial);
                            }
                            platform_specific::wayland::data_device::ActionInner::SetActions { preferred, accepted } => {
                                if let Some(offer) = self.state.dnd_offer.as_ref() {
                                    offer.offer.set_actions(accepted, preferred);
                                }
                            }
                        }
                    }
                }
            }

            // Send events cleared.
            sticky_exit_callback(
                IcedSctkEvent::MainEventsCleared,
                &self.state,
                &mut control_flow,
                &mut callback,
            );

            // redraw
            pending_redraws.dedup();
            for id in pending_redraws {
                sticky_exit_callback(
                    IcedSctkEvent::RedrawRequested(id.clone()),
                    &self.state,
                    &mut control_flow,
                    &mut callback,
                );
            }

            // commit changes made via actions
            for s in to_commit {
                s.1.commit();
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
