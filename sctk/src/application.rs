#[cfg(feature = "a11y")]
use crate::sctk_event::ActionRequestEvent;
use crate::{
    clipboard::Clipboard,
    commands::{layer_surface::get_layer_surface, window::get_window},
    error::{self, Error},
    event_loop::{control_flow::ControlFlow, proxy, SctkEventLoop},
    sctk_event::{
        DataSourceEvent, IcedSctkEvent, KeyboardEventVariant,
        LayerSurfaceEventVariant, PopupEventVariant, SctkEvent,
    },
    settings
};
use float_cmp::approx_eq;
use futures::{channel::mpsc, task, Future, FutureExt, StreamExt};
#[cfg(feature = "a11y")]
use iced_accessibility::{A11yId, accesskit::{NodeId, NodeBuilder}, A11yNode};
use iced_futures::{Executor, Runtime, core::{renderer::Style, widget::{operation::{self, OperationWrapper, focusable::focus}, tree, Tree, self, Operation}, layout::Limits, Widget, event::{Status, self}}, Subscription};
// use iced_native::{
//     application::{self, StyleSheet},
//     clipboard,
//     command::platform_specific::{
//         self,
//         wayland::{data_device::DndIcon, popup},
//     },
//     event::Status,
//     layout::Limits,
//     mouse::{self, Interaction},
//     widget::{operation::{self, focusable::{focus, find_focused}}, Tree, self},
//     Element, Renderer, Widget,
// };
use log::error;

use sctk::{
    reexports::client::{protocol::wl_surface::WlSurface, Proxy},
    seat::{keyboard::Modifiers, pointer::PointerEventKind},
};
use std::{collections::HashMap, ffi::c_void, hash::Hash, marker::PhantomData};
use wayland_backend::client::ObjectId;

use iced_graphics::{
    compositor, renderer, Viewport, Compositor,
    // window::{self, Compositor},
    // Color, Point, Viewport,
};
// use iced_native::user_interface::{self, UserInterface};
// use iced_native::window::Id as SurfaceId;
use itertools::Itertools;
// use iced_native::widget::Operation;
use std::mem::ManuallyDrop;
use raw_window_handle::{
    HasRawDisplayHandle, HasRawWindowHandle, RawDisplayHandle, RawWindowHandle,
    WaylandDisplayHandle, WaylandWindowHandle,
};
use iced_style::application::{StyleSheet, self};
use iced_runtime::{core::{mouse::Interaction, Element, Renderer, Point, Size, Color}, command::{platform_specific::{self, wayland::{data_device::DndIcon, popup}}, self}, window::Id as SurfaceId, Command, user_interface, UserInterface, system, clipboard, Debug, Program};
// use iced_native::widget::operation::OperationWrapper;

pub enum Event<Message> {
    /// A normal sctk event
    SctkEvent(IcedSctkEvent<Message>),
    /// TODO
    // (maybe we should also allow users to listen/react to those internal messages?)

    /// layer surface requests from the client
    LayerSurface(platform_specific::wayland::layer_surface::Action<Message>),
    /// window requests from the client
    Window(platform_specific::wayland::window::Action<Message>),
    /// popup requests from the client
    Popup(platform_specific::wayland::popup::Action<Message>),
    /// data device requests from the client
    DataDevice(platform_specific::wayland::data_device::Action<Message>),
    /// request sctk to set the cursor of the active pointer
    SetCursor(Interaction),
    /// Application Message
    Message(Message),
}

pub struct IcedSctkState;

/// An interactive, native cross-platform application.
///
/// This trait is the main entrypoint of Iced. Once implemented, you can run
/// your GUI application by simply calling [`run`]. It will run in
/// its own window.
///
/// An [`Application`] can execute asynchronous actions by returning a
/// [`Command`] in some of its methods.
///
/// When using an [`Application`] with the `debug` feature enabled, a debug view
/// can be toggled by pressing `F12`.
pub trait Application: Program
where
    <Self::Renderer as Renderer>::Theme: StyleSheet,
{
    /// The data needed to initialize your [`Application`].
    type Flags;

    /// Initializes the [`Application`] with the flags provided to
    /// [`run`] as part of the [`Settings`].
    ///
    /// Here is where you should return the initial state of your app.
    ///
    /// Additionally, you can return a [`Command`] if you need to perform some
    /// async action in the background on startup. This is useful if you want to
    /// load state from a file, perform an initial HTTP request, etc.
    fn new(flags: Self::Flags) -> (Self, Command<Self::Message>);

    /// Returns the current title of the [`Application`].
    ///
    /// This title can be dynamic! The runtime will automatically update the
    /// title of your application when necessary.
    fn title(&self) -> String;

    /// Returns the current [`Theme`] of the [`Application`].
    fn theme(&self) -> <Self::Renderer as Renderer>::Theme;

    /// Returns the [`Style`] variation of the [`Theme`].
    fn style(
        &self,
    ) -> <<Self::Renderer as Renderer>::Theme as StyleSheet>::Style {
        Default::default()
    }

    /// Returns the event `Subscription` for the current state of the
    /// application.
    ///
    /// The messages produced by the `Subscription` will be handled by
    /// [`update`](#tymethod.update).
    ///
    /// A `Subscription` will be kept alive as long as you keep returning it!
    ///
    /// By default, it returns an empty subscription.
    fn subscription(&self) -> Subscription<Self::Message> {
        Subscription::none()
    }

    /// Returns the scale factor of the [`Application`].
    ///
    /// It can be used to dynamically control the size of the UI at runtime
    /// (i.e. zooming).
    ///
    /// For instance, a scale factor of `2.0` will make widgets twice as big,
    /// while a scale factor of `0.5` will shrink them to half their size.
    ///
    /// By default, it returns `1.0`.
    fn scale_factor(&self) -> f64 {
        1.0
    }

    /// Returns whether the [`Application`] should be terminated.
    ///
    /// By default, it returns `false`.
    fn should_exit(&self) -> bool {
        false
    }

    /// TODO
    fn close_requested(&self, id: SurfaceId) -> Self::Message;
}

pub struct SurfaceDisplayWrapper<C: Compositor> {
    comp_surface: Option<<C as Compositor>::Surface>,
    backend: wayland_backend::client::Backend,
    wl_surface: WlSurface,
}

unsafe impl<C: Compositor> HasRawDisplayHandle for SurfaceDisplayWrapper<C> {
    fn raw_display_handle(&self) -> RawDisplayHandle {
        let mut display_handle = WaylandDisplayHandle::empty();
        display_handle.display = self.backend.display_ptr() as *mut _;
        RawDisplayHandle::Wayland(display_handle)
    }
}

unsafe impl<C: Compositor> HasRawWindowHandle for SurfaceDisplayWrapper<C> {
    fn raw_window_handle(&self) -> RawWindowHandle {
        let mut window_handle = WaylandWindowHandle::empty();
        window_handle.surface = self.wl_surface.id().as_ptr() as *mut _;
        RawWindowHandle::Wayland(window_handle)
    }
}

/// Runs an [`Application`] with an executor, compositor, and the provided
/// settings.
pub fn run<A, E, C>(
    settings: settings::Settings<A::Flags>,
    compositor_settings: C::Settings,
) -> Result<(), error::Error>
where
    A: Application + 'static,
    E: Executor + 'static,
    C: Compositor<Renderer = A::Renderer> + 'static,
    <A::Renderer as Renderer>::Theme: StyleSheet,
    A::Flags: Clone,
{
    let mut debug = Debug::new();
    debug.startup_started();

    let flags = settings.flags.clone();
    let exit_on_close_request = settings.exit_on_close_request;

    let mut event_loop = SctkEventLoop::<A::Message>::new(&settings)
        .expect("Failed to initialize the event loop");

    let (runtime, ev_proxy) = {
        let ev_proxy = event_loop.proxy();
        let executor = E::new().map_err(Error::ExecutorCreationFailed)?;

        (Runtime::new(executor, ev_proxy.clone()), ev_proxy)
    };

    let (application, init_command) = {
        let flags = flags;

        runtime.enter(|| A::new(flags))
    };

    let init_command = match settings.surface {
        settings::InitialSurface::LayerSurface(b) => {
            Command::batch(vec![init_command, get_layer_surface(b)])
        }
        settings::InitialSurface::XdgWindow(b) => {
            Command::batch(vec![init_command, get_window(b)])
        }
        settings::InitialSurface::None => init_command,
    };
    let wl_surface = event_loop
        .state
        .compositor_state
        .create_surface(&event_loop.state.queue_handle);

    // let (display, context, config, surface) = init_egl(&wl_surface, 100, 100);
    let backend = event_loop.state.connection.backend();
    let wrapper = SurfaceDisplayWrapper::<C> {
        comp_surface: None,
        backend: backend.clone(),
        wl_surface,
    };

    #[allow(unsafe_code)]
    let (compositor, renderer) =
        C::new(compositor_settings, Some(&wrapper)).unwrap();

    let auto_size_surfaces = HashMap::new();

    let surface_ids = Default::default();

    let (mut sender, receiver) = mpsc::unbounded::<IcedSctkEvent<A::Message>>();

    let compositor_surfaces = HashMap::new();
    let mut instance = Box::pin(run_instance::<A, E, C>(
        application,
        compositor,
        renderer,
        runtime,
        ev_proxy,
        debug,
        receiver,
        compositor_surfaces,
        surface_ids,
        auto_size_surfaces,
        // display,
        // context,
        // config,
        backend,
        init_command,
        exit_on_close_request,
    ));

    let mut context = task::Context::from_waker(task::noop_waker_ref());

    let _ = event_loop.run_return(move |event, _, control_flow| {
        if let ControlFlow::ExitWithCode(_) = control_flow {
            return;
        }

        sender.start_send(event).expect("Send event");

        let poll = instance.as_mut().poll(&mut context);

        *control_flow = match poll {
            task::Poll::Pending => ControlFlow::Wait,
            task::Poll::Ready(_) => ControlFlow::ExitWithCode(1),
        };
    });

    Ok(())
}

fn subscription_map<A, E, C>(e: A::Message) -> Event<A::Message>
where
    A: Application + 'static,
    E: Executor + 'static,
    C: Compositor<Renderer = A::Renderer> + 'static,
    <A::Renderer as Renderer>::Theme: StyleSheet,
{
    Event::SctkEvent(IcedSctkEvent::UserEvent(e))
}

// XXX Ashley careful, A, E, C must be exact same as in update, or the subscription map type will have a different hash
async fn run_instance<A, E, C>(
    mut application: A,
    mut compositor: C,
    mut renderer: A::Renderer,
    mut runtime: Runtime<E, proxy::Proxy<Event<A::Message>>, Event<A::Message>>,
    mut ev_proxy: proxy::Proxy<Event<A::Message>>,
    mut debug: Debug,
    mut receiver: mpsc::UnboundedReceiver<IcedSctkEvent<A::Message>>,
    mut compositor_surfaces: HashMap<SurfaceId, SurfaceDisplayWrapper<C>>,
    mut surface_ids: HashMap<ObjectId, SurfaceIdWrapper>,
    mut auto_size_surfaces: HashMap<SurfaceIdWrapper, (u32, u32, Limits, bool)>,
    backend: wayland_backend::client::Backend,
    init_command: Command<A::Message>,
    _exit_on_close_request: bool, // TODO Ashley
) -> Result<(), Error>
where
    A: Application + 'static,
    E: Executor + 'static,
    C: Compositor<Renderer = A::Renderer> + 'static,
    <A::Renderer as Renderer>::Theme: StyleSheet,
{
    let mut cache = user_interface::Cache::default();

    let mut states: HashMap<SurfaceId, State<A>> = HashMap::new();
    let mut interfaces = ManuallyDrop::new(HashMap::new());

    {
        run_command(
            &application,
            &mut cache,
            None,
            &mut renderer,
            init_command,
            &mut runtime,
            &mut ev_proxy,
            &mut debug,
            || compositor.fetch_information(),
            &mut auto_size_surfaces,
        );
    }
    runtime.track(application.subscription().map(subscription_map::<A, E, C>).into_recipes(),);

    let _mouse_interaction = Interaction::default();
    let mut sctk_events: Vec<SctkEvent> = Vec::new();
    #[cfg(feature = "a11y")]
    let mut a11y_events: Vec<crate::sctk_event::ActionRequestEvent> =
        Vec::new();
    #[cfg(feature = "a11y")]
    let mut a11y_enabled = false;
    #[cfg(feature = "a11y")]
    let mut adapters: HashMap<
        SurfaceId,
        crate::event_loop::adapter::IcedSctkAdapter,
    > = HashMap::new();

    let mut messages: Vec<A::Message> = Vec::new();
    let mut commands: Vec<Command<A::Message>> = Vec::new();
    debug.startup_finished();

    // let mut current_context_window = init_id_inner;

    let mut kbd_surface_id: Option<ObjectId> = None;
    let mut mods = Modifiers::default();
    let mut destroyed_surface_ids: HashMap<ObjectId, SurfaceIdWrapper> =
        Default::default();
    let mut simple_clipboard = Clipboard::unconnected();

    'main: while let Some(event) = receiver.next().await {
        match event {
            IcedSctkEvent::NewEvents(_) => {} // TODO Ashley: Seems to be ignored in iced_winit so i'll ignore for now
            IcedSctkEvent::UserEvent(message) => {
                messages.push(message);
            }
            IcedSctkEvent::SctkEvent(event) => {
                sctk_events.push(event.clone());
                match event {
                    SctkEvent::SeatEvent { .. } => {} // TODO Ashley: handle later possibly if multiseat support is wanted
                    SctkEvent::PointerEvent {
                        variant,
                        ..
                    } => {
                        let (state, _native_id) = match surface_ids
                            .get(&variant.surface.id())
                            .and_then(|id| states.get_mut(&id.inner()).map(|state| (state, id)))
                        {
                            Some(s) => s,
                            None => continue,
                        };
                        match variant.kind {
                            PointerEventKind::Enter { .. } => {
                                state.set_cursor_position(Point::new(
                                    variant.position.0 as f32,
                                    variant.position.1 as f32,
                                ));
                            }
                            PointerEventKind::Leave { .. } => {
                                state.set_cursor_position(Point::new(-1.0, -1.0));
                            }
                            PointerEventKind::Motion { .. } => {
                                state.set_cursor_position(Point::new(
                                    variant.position.0 as f32,
                                    variant.position.1 as f32,
                                ));
                            }
                            PointerEventKind::Press { .. }
                            | PointerEventKind::Release { .. }
                            | PointerEventKind::Axis { .. } => {}
                        }
                    }
                    SctkEvent::KeyboardEvent { variant, .. } => match variant {
                        KeyboardEventVariant::Leave(_) => {
                            kbd_surface_id.take();
                        }
                        KeyboardEventVariant::Enter(object_id) => {
                            kbd_surface_id.replace(object_id.id());
                        }
                        KeyboardEventVariant::Press(_)
                        | KeyboardEventVariant::Release(_)
                        | KeyboardEventVariant::Repeat(_) => {}
                        KeyboardEventVariant::Modifiers(mods) => {
                            if let Some(state) = kbd_surface_id
                                .as_ref()
                                .and_then(|id| surface_ids.get(id))
                                .and_then(|id| states.get_mut(&id.inner()))
                            {
                                state.modifiers = mods;
                            }
                        }
                    },
                    SctkEvent::WindowEvent { variant, id } => match variant {
                        crate::sctk_event::WindowEventVariant::Created(id, native_id) => {
                            surface_ids.insert(id, SurfaceIdWrapper::Window(native_id));
                        }
                        crate::sctk_event::WindowEventVariant::Close => {
                            if let Some(surface_id) = surface_ids.remove(&id.id()) {
                                // drop(compositor_surfaces.remove(&surface_id.inner()));
                                interfaces.remove(&surface_id.inner());
                                states.remove(&surface_id.inner());
                                messages.push(application.close_requested(surface_id.inner()));
                                destroyed_surface_ids.insert(id.id(), surface_id);
                                // if exit_on_close_request && surface_id == init_id {
                                //     break 'main;
                                // }
                            }
                        }
                        crate::sctk_event::WindowEventVariant::WmCapabilities(_)
                        | crate::sctk_event::WindowEventVariant::ConfigureBounds { .. } => {}
                        crate::sctk_event::WindowEventVariant::Configure(
                            configure,
                            wl_surface,
                            first,
                        ) => {
                            if let Some(id) = surface_ids.get(&id.id()) {
                                compositor_surfaces.entry(id.inner()).or_insert_with(|| {
                                     let mut wrapper = SurfaceDisplayWrapper {
                                         comp_surface: None,
                                         backend: backend.clone(),
                                         wl_surface
                                     };
                                     if matches!(simple_clipboard.state,  crate::clipboard::State::Unavailable) {
                                        if let RawDisplayHandle::Wayland(handle) = wrapper.raw_display_handle() {
                                            assert!(!handle.display.is_null());
                                            simple_clipboard = unsafe { Clipboard::connect(handle.display as *mut c_void) };
                                        }
                                     }
                                     let c_surface = compositor.create_surface(&wrapper, configure.new_size.0.unwrap().get(), configure.new_size.1.unwrap().get());
                                     wrapper.comp_surface.replace(c_surface);
                                     wrapper
                                 });
                                if first {
                                    let state = State::new(&application, *id);

                                    let user_interface = build_user_interface(
                                        &application,
                                        user_interface::Cache::default(),
                                        &mut renderer,
                                        state.logical_size(),
                                        &state.title,
                                        &mut debug,
                                        *id,
                                        &mut auto_size_surfaces,
                                        &mut ev_proxy
                                    );
                                    states.insert(id.inner(), state);
                                    interfaces.insert(id.inner(), user_interface);
                                }
                                if let Some(state) = states.get_mut(&id.inner()) {
                                    state.set_logical_size(configure.new_size.0.unwrap().get() as f64 , configure.new_size.1.unwrap().get() as f64);
                                }
                            }
                        }
                    },
                    SctkEvent::LayerSurfaceEvent { variant, id } => match variant {
                        LayerSurfaceEventVariant::Created(id, native_id) => {
                            surface_ids.insert(id, SurfaceIdWrapper::LayerSurface(native_id));
                        }
                        LayerSurfaceEventVariant::Done => {
                            if let Some(surface_id) = surface_ids.remove(&id.id()) {
                                drop(compositor_surfaces.remove(&surface_id.inner()));
                                interfaces.remove(&surface_id.inner());
                                states.remove(&surface_id.inner());
                                messages.push(application.close_requested(surface_id.inner()));
                                destroyed_surface_ids.insert(id.id(), surface_id);
                                // if exit_on_close_request && surface_id == init_id {
                                //     break 'main;
                                // }
                            }
                        }
                        LayerSurfaceEventVariant::Configure(configure, wl_surface, first) => {
                            if let Some(id) = surface_ids.get(&id.id()) {
                                compositor_surfaces.entry(id.inner()).or_insert_with(|| {
                                     let mut wrapper = SurfaceDisplayWrapper {
                                         comp_surface: None,
                                         backend: backend.clone(),
                                         wl_surface
                                     };
                                     if matches!(simple_clipboard.state,  crate::clipboard::State::Unavailable) {
                                        if let RawDisplayHandle::Wayland(handle) = wrapper.raw_display_handle() {
                                            assert!(!handle.display.is_null());
                                            simple_clipboard = unsafe { Clipboard::connect(handle.display as *mut c_void) };
                                        }
                                     }
                                     let mut c_surface = compositor.create_surface(&wrapper, configure.new_size.0, configure.new_size.1);
                                     compositor.configure_surface(&mut c_surface, configure.new_size.0, configure.new_size.1);
                                     wrapper.comp_surface.replace(c_surface);
                                     wrapper
                                });
                                if first {
                                    let state = State::new(&application, *id);

                                    let user_interface = build_user_interface(
                                        &application,
                                        user_interface::Cache::default(),
                                        &mut renderer,
                                        state.logical_size(),
                                        &state.title,
                                        &mut debug,
                                        *id,
                                        &mut auto_size_surfaces,
                                        &mut ev_proxy
                                    );
                                    states.insert(id.inner(), state);
                                    interfaces.insert(id.inner(), user_interface);
                                }
                                if let Some(state) = states.get_mut(&id.inner()) {
                                    state.set_logical_size(
                                        configure.new_size.0 as f64,
                                        configure.new_size.1 as f64,
                                    );
                                }
                            }
                        }
                    },
                    SctkEvent::PopupEvent {
                        variant,
                        toplevel_id: _,
                        parent_id: _,
                        id,
                    } => match variant {
                        PopupEventVariant::Created(id, native_id) => {
                            surface_ids.insert(id, SurfaceIdWrapper::Popup(native_id));
                        }
                        PopupEventVariant::Done => {
                            if let Some(surface_id) = surface_ids.remove(&id.id()) {
                                drop(compositor_surfaces.remove(&surface_id.inner()));
                                interfaces.remove(&surface_id.inner());
                                states.remove(&surface_id.inner());
                                messages.push(application.close_requested(surface_id.inner()));
                                destroyed_surface_ids.insert(id.id(), surface_id);
                            }
                        }
                        PopupEventVariant::WmCapabilities(_) => {}
                        PopupEventVariant::Configure(configure, wl_surface, first) => {
                            if let Some(id) = surface_ids.get(&id.id()) {
                               compositor_surfaces.entry(id.inner()).or_insert_with(|| {
                                     let mut wrapper = SurfaceDisplayWrapper {
                                         comp_surface: None,
                                         backend: backend.clone(),
                                         wl_surface
                                     };
                                     let c_surface = compositor.create_surface(&wrapper, configure.width as u32, configure.height as u32);
                                     wrapper.comp_surface.replace(c_surface);
                                     wrapper
                                });
                                if first {
                                    let state = State::new(&application, *id);

                                    let user_interface = build_user_interface(
                                        &application,
                                        user_interface::Cache::default(),
                                        &mut renderer,
                                        state.logical_size(),
                                        &state.title,
                                        &mut debug,
                                        *id,
                                        &mut auto_size_surfaces,
                                        &mut ev_proxy
                                    );
                                    states.insert(id.inner(), state);
                                    interfaces.insert(id.inner(), user_interface);
                                }
                                if let Some(state) = states.get_mut(&id.inner()) {
                                    state.set_logical_size(
                                        configure.width as f64,
                                        configure.height as f64,
                                    );
                                }
                            }
                        }
                        PopupEventVariant::RepositionionedPopup { .. } => {}
                        PopupEventVariant::Size(width, height) => {
                            if let Some(id) = surface_ids.get(&id.id()) {
                                if let Some(state) = states.get_mut(&id.inner()) {
                                    state.set_logical_size(
                                        width as f64,
                                        height as f64,
                                    );
                                }
                            }
                        },
                    },
                    // TODO forward these events to an application which requests them?
                    SctkEvent::NewOutput { .. } => {
                    }
                    SctkEvent::UpdateOutput { .. } => {
                    }
                    SctkEvent::RemovedOutput( ..) => {
                    }
                    SctkEvent::Frame(_) => {
                        // TODO if animations are running, request redraw here?
                    },
                    SctkEvent::ScaleFactorChanged {
                        factor,
                        id,
                        inner_size: _,
                    } => {
                        if let Some(state) = surface_ids
                            .get(&id.id())
                            .and_then(|id| states.get_mut(&id.inner()))
                        {
                            state.set_scale_factor(factor);
                        }
                    }
                    SctkEvent::DataSource(DataSourceEvent::DndFinished) | SctkEvent::DataSource(DataSourceEvent::DndCancelled)=> {
                        surface_ids.retain(|id, surface_id| {
                            match surface_id {
                                SurfaceIdWrapper::Dnd(inner) => {
                                    drop(compositor_surfaces.remove(&inner));
                                    interfaces.remove(inner);
                                    states.remove(inner);
                                    destroyed_surface_ids.insert(id.clone(), *surface_id);
                                    false
                                },
                                _ => true,
                            }
                        })
                    }
                    _ => {}
                }
            }
            IcedSctkEvent::DndSurfaceCreated(
                wl_surface,
                dnd_icon,
                origin_id,
            ) => {
                // if the surface is meant to be drawn as a custom widget by the
                // application, we should treat it like any other surfaces
                //
                // TODO if the surface is meant to be drawn by a widget that implements
                // draw_dnd_icon, we should mark it and not pass it to the view method
                // of the Application
                //
                // Dnd Surfaces are only drawn once

                let id = wl_surface.id();
                let (native_id, e) = match dnd_icon {
                    DndIcon::Custom(id) => {
                        let mut e = application.view(id);
                        let state = e.as_widget().state();
                        let tag = e.as_widget().tag();
                        let mut tree = Tree {
                            id: e.as_widget().id(),
                            tag,
                            state,
                            children: e.as_widget().children(),
                        };
                        e.as_widget_mut().diff(&mut tree);
                        (id, e)
                    }
                    DndIcon::Widget(id, widget_state) => {
                        let mut e = application.view(id);
                        let mut tree = Tree {
                            id: e.as_widget().id(),
                            tag: e.as_widget().tag(),
                            state: tree::State::Some(widget_state),
                            children: e.as_widget().children(),
                        };
                        e.as_widget_mut().diff(&mut tree);
                        (id, e)
                    }
                };
                let node =
                    Widget::layout(e.as_widget(), &renderer, &Limits::NONE);
                let bounds = node.bounds();
                let w = bounds.width.ceil() as u32;
                let h = bounds.height.ceil() as u32;
                if w == 0 || h == 0 {
                    error!("Dnd surface has zero size, ignoring");
                    continue;
                }
                let parent_size = states
                    .get(&origin_id)
                    .map(|s| s.logical_size())
                    .unwrap_or_else(|| Size::new(1024.0, 1024.0));
                if w > parent_size.width as u32 || h > parent_size.height as u32
                {
                    error!("Dnd surface is too large, ignoring");
                    continue;
                }
                let mut wrapper = SurfaceDisplayWrapper {
                    comp_surface: None,
                    backend: backend.clone(),
                    wl_surface,
                };
                let mut c_surface = compositor.create_surface(&wrapper, w, h);
                compositor.configure_surface(&mut c_surface, w, h);
                let mut state =
                    State::new(&application, SurfaceIdWrapper::Dnd(native_id));
                state.set_logical_size(w as f64, h as f64);
                let mut user_interface = build_user_interface(
                    &application,
                    user_interface::Cache::default(),
                    &mut renderer,
                    state.logical_size(),
                    &state.title,
                    &mut debug,
                    SurfaceIdWrapper::Dnd(native_id),
                    &mut auto_size_surfaces,
                    &mut ev_proxy,
                );
                state.synchronize(&application);

                // just draw here immediately and never again for dnd icons
                // TODO handle scale factor?
                let _new_mouse_interaction = user_interface.draw(
                    &mut renderer,
                    state.theme(),
                    &Style {
                        text_color: state.text_color(),
                    },
                    state.cursor_position(),
                );
                let _ = compositor.present(
                    &mut renderer,
                    &mut c_surface,
                    state.viewport(),
                    Color::TRANSPARENT,
                    &debug.overlay(),
                );
                wrapper.comp_surface.replace(c_surface);
                surface_ids.insert(id, SurfaceIdWrapper::Dnd(native_id));
                compositor_surfaces
                    .entry(native_id)
                    .or_insert_with(move || wrapper);
                states.insert(native_id, state);
                interfaces.insert(native_id, user_interface);
            }
            IcedSctkEvent::MainEventsCleared => {
                let mut i = 0;
                while i < sctk_events.len() {
                    let remove = matches!(
                        sctk_events[i],
                        SctkEvent::NewOutput { .. }
                            | SctkEvent::UpdateOutput { .. }
                            | SctkEvent::RemovedOutput(_)
                    );
                    if remove {
                        let event = sctk_events.remove(i);
                        for native_event in event.to_native(
                            &mut mods,
                            &surface_ids,
                            &destroyed_surface_ids,
                        ) {
                            runtime.broadcast(native_event, Status::Ignored);
                        }
                    } else {
                        i += 1;
                    }
                }

                if surface_ids.is_empty() && !messages.is_empty() {
                    // Update application
                    let pure_states: HashMap<_, _> =
                        ManuallyDrop::into_inner(interfaces)
                            .drain()
                            .map(|(id, interface)| (id, interface.into_cache()))
                            .collect();

                    // Update application
                    update::<A, E, C>(
                        &mut application,
                        &mut cache,
                        None,
                        &mut renderer,
                        &mut runtime,
                        &mut ev_proxy,
                        &mut debug,
                        &mut messages,
                        || compositor.fetch_information(),
                        &mut auto_size_surfaces,
                    );

                    interfaces = ManuallyDrop::new(build_user_interfaces(
                        &application,
                        &mut renderer,
                        &mut debug,
                        &states,
                        pure_states,
                        &mut auto_size_surfaces,
                        &mut ev_proxy,
                    ));

                    if application.should_exit() {
                        break 'main;
                    }
                } else {
                    // TODO ensure that the surface_ids are
                    let mut needs_redraw = false;
                    for (object_id, surface_id) in &surface_ids {
                        if matches!(surface_id, SurfaceIdWrapper::Dnd(_)) {
                            continue;
                        }
                        let mut filtered_sctk = Vec::with_capacity(sctk_events.len());

                        let mut i = 0;
                        while i < sctk_events.len() {
                            let has_kbd_focus =
                                kbd_surface_id.as_ref() == Some(object_id);
                            if event_is_for_surface(
                                &sctk_events[i],
                                object_id,
                                has_kbd_focus,
                            ) {
                                filtered_sctk.push(sctk_events.remove(i));
                            } else {
                                i += 1;
                            }
                        }
                        let mut has_events = !sctk_events.is_empty();

                        let cursor_position =
                            match states.get(&surface_id.inner()) {
                                Some(s) => s.cursor_position(),
                                None => continue,
                            };
                        debug.event_processing_started();
                        let mut native_events: Vec<_> = filtered_sctk
                            .into_iter()
                            .flat_map(|e| {
                                e.to_native(
                                    &mut mods,
                                    &surface_ids,
                                    &destroyed_surface_ids,
                                )
                            })
                            .collect();

                        #[cfg(feature = "a11y")]
                        {
                            let mut filtered_a11y =
                                Vec::with_capacity(a11y_events.len());
                            while i < a11y_events.len() {
                                if a11y_events[i].surface_id == *object_id {
                                    filtered_a11y.push(a11y_events.remove(i));
                                } else {
                                    i += 1;
                                }
                            }                        
                            native_events.extend(filtered_a11y.into_iter().map(|e| {
                                event::Event::A11y(widget::Id::from(u128::from(e.request.target.0) as u64), e.request)
                            }));
                        }
                        let has_events =
                            has_events || !native_events.is_empty();

                        let (interface_state, statuses) = {
                            let user_interface = interfaces
                                .get_mut(&surface_id.inner())
                                .unwrap();
                            user_interface.update(
                                native_events.as_slice(),
                                cursor_position,
                                &mut renderer,
                                &mut simple_clipboard,
                                &mut messages,
                            )
                        };
                        debug.event_processing_finished();
                        for (event, status) in
                            native_events.into_iter().zip(statuses.into_iter())
                        {
                            runtime.broadcast(event, status);
                        }

                        if let Some((w, h, limits, dirty)) =
                            auto_size_surfaces.remove(surface_id)
                        {
                            if dirty {
                                let state =
                                    match states.get_mut(&surface_id.inner()) {
                                        Some(s) => s,
                                        None => continue,
                                    };
                                state.set_logical_size(w as f64, h as f64);
                            }
                            auto_size_surfaces
                                .insert(*surface_id, (w, h, limits, false));
                        }

                        // TODO ASHLEY if event is a configure which isn't a new size and has no other changes, don't redraw
                        if has_events
                            || !messages.is_empty()
                            || matches!(
                                interface_state,
                                user_interface::State::Outdated
                            )
                        {
                            needs_redraw = true;
                            ev_proxy.send_event(Event::SctkEvent(
                                IcedSctkEvent::RedrawRequested(
                                    object_id.clone(),
                                ),
                            ));
                        }
                    }
                    if needs_redraw {
                        let mut pure_states: HashMap<_, _> =
                            ManuallyDrop::into_inner(interfaces)
                                .drain()
                                .map(|(id, interface)| {
                                    (id, interface.into_cache())
                                })
                                .collect();

                        for surface_id in surface_ids.values() {
                            let state =
                                match states.get_mut(&surface_id.inner()) {
                                    Some(s) => s,
                                    None => continue,
                                };
                            let mut cache =
                                match pure_states.remove(&surface_id.inner()) {
                                    Some(cache) => cache,
                                    None => user_interface::Cache::default(),
                                };

                            // Update application
                            update::<A, E, C>(
                                &mut application,
                                &mut cache,
                                Some(state),
                                &mut renderer,
                                &mut runtime,
                                &mut ev_proxy,
                                &mut debug,
                                &mut messages,
                                || compositor.fetch_information(),
                                &mut auto_size_surfaces,
                            );

                            pure_states.insert(surface_id.inner(), cache);

                            // Update state
                            state.synchronize(&application);

                            if application.should_exit() {
                                break 'main;
                            }
                        }
                        interfaces = ManuallyDrop::new(build_user_interfaces(
                            &application,
                            &mut renderer,
                            &mut debug,
                            &states,
                            pure_states,
                            &mut auto_size_surfaces,
                            &mut ev_proxy,
                        ));
                    }
                }
                sctk_events.clear();
                // clear the destroyed surfaces after they have been handled
                destroyed_surface_ids.clear();
            }
            IcedSctkEvent::RedrawRequested(object_id) => {
                if let Some((
                    native_id,
                    Some(wrapper),
                    Some(mut user_interface),
                    Some(state),
                )) = surface_ids.get(&object_id).and_then(|id| {
                    if matches!(id, SurfaceIdWrapper::Dnd(_)) {
                        return None;
                    }
                    let surface = compositor_surfaces.get_mut(&id.inner());
                    let interface = interfaces.remove(&id.inner());
                    let state = states.get_mut(&id.inner());
                    Some((*id, surface, interface, state))
                }) {
                    debug.render_started();
                    #[cfg(feature = "a11y")]
                    if let Some(Some(adapter)) = a11y_enabled.then(|| adapters.get_mut(&native_id.inner())) {
                        use iced_accessibility::{A11yTree, accesskit::{TreeUpdate, Tree, Role}};
                        // TODO send a11y tree
                        let child_tree = user_interface.a11y_nodes(state.cursor_position());
                        let mut root = NodeBuilder::new(Role::Window);
                        root.set_name(state.title().to_string());
                        let window_tree = A11yTree::node_with_child_tree(A11yNode::new(root, adapter.id), child_tree);
                        let tree = Tree::new(NodeId(adapter.id));
                        let mut current_operation = Some(Box::new(OperationWrapper::Id(Box::new(operation::focusable::find_focused()))));
                        let mut focus = None;
                        while let Some(mut operation) = current_operation.take() {
                            user_interface.operate(&renderer, operation.as_mut());
        
                            match operation.finish() {
                                operation::Outcome::None => {
                                }
                                operation::Outcome::Some(message) => {
                                    match message {
                                        operation::OperationOutputWrapper::Message(m) => {
                                            unimplemented!();
                                        }
                                        operation::OperationOutputWrapper::Id(id) => {
                                            focus = Some(A11yId::from(id));
                                        },
                                    }
                                   
                                }
                                operation::Outcome::Chain(mut next) => {
                                    current_operation = Some(Box::new(OperationWrapper::Wrapper(next)));
                                }
                            }
                        }
                        log::debug!("focus: {:?}\ntree root: {:?}\n children: {:?}", &focus, window_tree.root().iter().map(|n| (n.node().role(), n.id())).collect::<Vec<_>>(), window_tree.children().iter().map(|n| (n.node().role(), n.id())).collect::<Vec<_>>());
                        let focus = focus
                            .filter(|f_id| window_tree.contains(f_id))
                            .map(|id| id.into());
                        adapter.adapter.update(TreeUpdate {
                            nodes: window_tree.into(),
                            tree: Some(tree),
                            focus,
                        });
                    }
                    let comp_surface = match wrapper.comp_surface.as_mut() {
                        Some(s) => s,
                        None => continue,
                    };

                    if state.viewport_changed() {
                        let physical_size = state.physical_size();
                        let logical_size = state.logical_size();
                        compositor.configure_surface(
                            comp_surface,
                            physical_size.width,
                            physical_size.height,
                        );

                        debug.layout_started();
                        user_interface = user_interface
                            .relayout(logical_size, &mut renderer);
                        debug.layout_finished();

                        debug.draw_started();
                        let new_mouse_interaction = user_interface.draw(
                            &mut renderer,
                            state.theme(),
                            &Style {
                                text_color: state.text_color(),
                            },
                            state.cursor_position(),
                        );
                        debug.draw_finished();
                        ev_proxy.send_event(Event::SetCursor(
                            new_mouse_interaction,
                        ));

                        let _ = interfaces
                            .insert(native_id.inner(), user_interface);

                        state.viewport_changed = false;
                    } else {
                        debug.draw_started();
                        let new_mouse_interaction = user_interface.draw(
                            &mut renderer,
                            state.theme(),
                            &Style {
                                text_color: state.text_color(),
                            },
                            state.cursor_position(),
                        );
                        debug.draw_finished();
                        ev_proxy.send_event(Event::SetCursor(
                            new_mouse_interaction,
                        ));
                        interfaces.insert(native_id.inner(), user_interface);
                    }

                    let _ = compositor.present(
                        &mut renderer,
                        comp_surface,
                        state.viewport(),
                        state.background_color(),
                        &debug.overlay(),
                    );

                    debug.render_finished();
                }
            }
            IcedSctkEvent::RedrawEventsCleared => {
                // TODO
            }
            IcedSctkEvent::LoopDestroyed => todo!(),
            #[cfg(feature = "a11y")]
            IcedSctkEvent::A11yEvent(ActionRequestEvent {
                surface_id,
                request,
            }) => {
                use iced_accessibility::accesskit::Action;  
                match request.action {
                    Action::Default => {
                        // TODO default operation?
                        // messages.push(focus(request.target.into()));
                        a11y_events.push(ActionRequestEvent { surface_id, request });
                    },
                    Action::Focus => {
                        commands.push(Command::widget(focus(widget::Id::from(u128::from(request.target.0) as u64))));
                    },
                    Action::Blur => todo!(),
                    Action::Collapse => todo!(),
                    Action::Expand => todo!(),
                    Action::CustomAction => todo!(),
                    Action::Decrement => todo!(),
                    Action::Increment => todo!(),
                    Action::HideTooltip => todo!(),
                    Action::ShowTooltip => todo!(),
                    Action::InvalidateTree => todo!(),
                    Action::LoadInlineTextBoxes => todo!(),
                    Action::ReplaceSelectedText => todo!(),
                    Action::ScrollBackward => todo!(),
                    Action::ScrollDown => todo!(),
                    Action::ScrollForward => todo!(),
                    Action::ScrollLeft => todo!(),
                    Action::ScrollRight => todo!(),
                    Action::ScrollUp => todo!(),
                    Action::ScrollIntoView => todo!(),
                    Action::ScrollToPoint => todo!(),
                    Action::SetScrollOffset => todo!(),
                    Action::SetTextSelection => todo!(),
                    Action::SetSequentialFocusNavigationStartingPoint => todo!(),
                    Action::SetValue => todo!(),
                    Action::ShowContextMenu => todo!(),
                }
            }
            #[cfg(feature = "a11y")]
            IcedSctkEvent::A11yEnabled => {
                a11y_enabled = true;
            }
            #[cfg(feature = "a11y")]
            IcedSctkEvent::A11ySurfaceCreated(surface_id, adapter) => {
                adapters.insert(surface_id.inner(), adapter);
            }
        }
    }

    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SurfaceIdWrapper {
    LayerSurface(SurfaceId),
    Window(SurfaceId),
    Popup(SurfaceId),
    Dnd(SurfaceId),
}

impl SurfaceIdWrapper {
    pub fn inner(&self) -> SurfaceId {
        match self {
            SurfaceIdWrapper::LayerSurface(id) => *id,
            SurfaceIdWrapper::Window(id) => *id,
            SurfaceIdWrapper::Popup(id) => *id,
            SurfaceIdWrapper::Dnd(id) => *id,
        }
    }
}

/// Builds a [`UserInterface`] for the provided [`Application`], logging
/// [`struct@Debug`] information accordingly.
pub fn build_user_interface<'a, A: Application>(
    application: &'a A,
    cache: user_interface::Cache,
    renderer: &mut A::Renderer,
    size: Size,
    title: &str,
    debug: &mut Debug,
    id: SurfaceIdWrapper,
    auto_size_surfaces: &mut HashMap<
        SurfaceIdWrapper,
        (u32, u32, Limits, bool),
    >,
    ev_proxy: &mut proxy::Proxy<Event<A::Message>>,
) -> UserInterface<'a, A::Message, A::Renderer>
where
    <A::Renderer as Renderer>::Theme: StyleSheet,
{
    debug.view_started();
    let mut view = application.view(id.inner());
    debug.view_finished();

    let size = if let Some((prev_w, prev_h, limits, dirty)) =
        auto_size_surfaces.remove(&id)
    {
        let view = view.as_widget_mut();
        let _state = view.state();
        // TODO would it be ok to diff against the current cache?
        view.diff(&mut Tree::empty());
        let bounds = view.layout(renderer, &limits).bounds().size();
        let (w, h) = (bounds.width.ceil() as u32, bounds.height.ceil() as u32);
        let dirty = dirty || w != prev_w || h != prev_h;
        auto_size_surfaces.insert(id, (w, h, limits, dirty));
        if dirty {
            match id {
                SurfaceIdWrapper::LayerSurface(inner) => {
                    ev_proxy.send_event(
                        Event::LayerSurface(
                            command::platform_specific::wayland::layer_surface::Action::Size { id: inner, width: Some(w), height: Some(h) },
                        )
                    );
                }
                SurfaceIdWrapper::Window(inner) => {
                    ev_proxy.send_event(
                        Event::Window(
                            command::platform_specific::wayland::window::Action::Size { id: inner, width: w, height: h },
                        )
                    );
                }
                SurfaceIdWrapper::Popup(inner) => {
                    ev_proxy.send_event(
                        Event::Popup(
                            command::platform_specific::wayland::popup::Action::Size { id: inner, width: w, height: h },
                        )
                    );
                }
                SurfaceIdWrapper::Dnd(_) => {}
            };
        }

        Size::new(w as f32, h as f32)
    } else {
        size
    };

    debug.layout_started();
    let user_interface = UserInterface::build(view, size, cache, renderer);
    debug.layout_finished();

    user_interface
}

/// The state of a surface created by the application [`Application`].
#[allow(missing_debug_implementations)]
pub struct State<A: Application>
where
    <A::Renderer as Renderer>::Theme: application::StyleSheet,
{
    pub(crate) id: SurfaceIdWrapper,
    title: String,
    scale_factor: f64,
    pub(crate) viewport: Viewport,
    viewport_changed: bool,
    cursor_position: Point,
    modifiers: Modifiers,
    theme: <A::Renderer as Renderer>::Theme,
    appearance: application::Appearance,
    application: PhantomData<A>,
}

impl<A: Application> State<A>
where
    <A::Renderer as Renderer>::Theme: application::StyleSheet,
{
    /// Creates a new [`State`] for the provided [`Application`]
    pub fn new(application: &A, id: SurfaceIdWrapper) -> Self {
        let title = application.title();
        let scale_factor = application.scale_factor();
        let theme = application.theme();
        let appearance = theme.appearance(&application.style());
        let viewport = Viewport::with_physical_size(Size::new(1, 1), 1.0);

        Self {
            id,
            title,
            scale_factor,
            viewport,
            viewport_changed: true,
            // TODO: Encode cursor availability in the type-system
            cursor_position: Point::new(-1.0, -1.0),
            modifiers: Modifiers::default(),
            theme,
            appearance,
            application: PhantomData,
        }
    }

    /// Returns the current [`Viewport`] of the [`State`].
    pub fn viewport(&self) -> &Viewport {
        &self.viewport
    }

    /// Returns the current title of the [`State`].
    pub fn title(&self) -> &str {
        &self.title
    }

    /// TODO
    pub fn viewport_changed(&self) -> bool {
        self.viewport_changed
    }

    /// Returns the physical [`Size`] of the [`Viewport`] of the [`State`].
    pub fn physical_size(&self) -> Size<u32> {
        self.viewport.physical_size()
    }

    /// Returns the logical [`Size`] of the [`Viewport`] of the [`State`].
    pub fn logical_size(&self) -> Size<f32> {
        self.viewport.logical_size()
    }

    /// Sets the logical [`Size`] of the [`Viewport`] of the [`State`].
    pub fn set_logical_size(&mut self, w: f64, h: f64) {
        let old_size = self.viewport.logical_size();
        if !approx_eq!(f32, w as f32, old_size.width, ulps = 2)
            || !approx_eq!(f32, h as f32, old_size.height, ulps = 2)
        {
            self.viewport_changed = true;
            self.viewport = Viewport::with_physical_size(
                Size {
                    width: (w * self.scale_factor) as u32,
                    height: (h * self.scale_factor) as u32,
                },
                self.scale_factor,
            );
        }
    }

    /// Returns the current scale factor of the [`Viewport`] of the [`State`].
    pub fn scale_factor(&self) -> f64 {
        self.viewport.scale_factor()
    }

    pub fn set_scale_factor(&mut self, scale_factor: f64) {
        if approx_eq!(f64, scale_factor, self.scale_factor, ulps = 2) {
            self.viewport_changed = true;
            let logical_size = self.viewport.logical_size();
            self.viewport = Viewport::with_physical_size(
                Size {
                    width: (logical_size.width as f64 * scale_factor) as u32,
                    height: (logical_size.height as f64 * scale_factor) as u32,
                },
                self.scale_factor,
            );
        }
    }

    /// Returns the current cursor position of the [`State`].
    pub fn cursor_position(&self) -> Point {
        self.cursor_position
    }

    /// Returns the current keyboard modifiers of the [`State`].
    pub fn modifiers(&self) -> Modifiers {
        self.modifiers
    }

    /// Returns the current theme of the [`State`].
    pub fn theme(&self) -> &<A::Renderer as Renderer>::Theme {
        &self.theme
    }

    /// Returns the current background [`Color`] of the [`State`].
    pub fn background_color(&self) -> Color {
        self.appearance.background_color
    }

    /// Returns the current text [`Color`] of the [`State`].
    pub fn text_color(&self) -> Color {
        self.appearance.text_color
    }

    pub fn set_cursor_position(&mut self, p: Point) {
        self.cursor_position = p;
    }

    fn synchronize(&mut self, application: &A) {
        // Update theme and appearance
        self.theme = application.theme();
        self.appearance = self.theme.appearance(&application.style());
    }
}

// XXX Ashley careful, A, E, C must be exact same as in run_instance, or the subscription map type will have a different hash
/// Updates an [`Application`] by feeding it the provided messages, spawning any
/// resulting [`Command`], and tracking its [`Subscription`]
pub(crate) fn update<A, E, C>(
    application: &mut A,
    cache: &mut user_interface::Cache,
    state: Option<&State<A>>,
    renderer: &mut A::Renderer,
    runtime: MyRuntime<E, A::Message>,
    proxy: &mut proxy::Proxy<Event<A::Message>>,
    debug: &mut Debug,
    messages: &mut Vec<A::Message>,
    graphics_info: impl FnOnce() -> compositor::Information + Copy,
    auto_size_surfaces: &mut HashMap<
        SurfaceIdWrapper,
        (u32, u32, Limits, bool),
    >,
) where
    A: Application + 'static,
    E: Executor + 'static,
    C: iced_graphics::Compositor<Renderer = A::Renderer> + 'static,
    <A::Renderer as Renderer>::Theme: StyleSheet,
{
    for message in messages.drain(..) {
        debug.log_message(&message);

        debug.update_started();
        let command = runtime.enter(|| application.update(message));
        debug.update_finished();

        run_command(
            application,
            cache,
            state,
            renderer,
            command,
            runtime,
            proxy,
            debug,
            graphics_info,
            auto_size_surfaces,
        );
    }

    runtime.track(application.subscription().map(subscription_map::<A, E, C>).into_recipes(),);
}

type MyRuntime<'a, E, M> = &'a mut Runtime<E, proxy::Proxy<Event<M>>, Event<M>>;

/// Runs the actions of a [`Command`].
fn run_command<A, E>(
    application: &A,
    cache: &mut user_interface::Cache,
    state: Option<&State<A>>,
    renderer: &mut A::Renderer,
    command: Command<A::Message>,
    runtime: MyRuntime<E, A::Message>,
    proxy: &mut proxy::Proxy<Event<A::Message>>,
    debug: &mut Debug,
    _graphics_info: impl FnOnce() -> compositor::Information + Copy,
    auto_size_surfaces: &mut HashMap<
        SurfaceIdWrapper,
        (u32, u32, Limits, bool),
    >,
) where
    A: Application,
    E: Executor,
    <A::Renderer as Renderer>::Theme: StyleSheet,
{
    for action in command.actions() {
        match action {
            command::Action::Future(future) => {
                runtime
                    .spawn(Box::pin(future.map(|e| {
                        Event::SctkEvent(IcedSctkEvent::UserEvent(e))
                    })));
            }
            command::Action::Clipboard(action) => match action {
                clipboard::Action::Read(..) => {
                    todo!();
                }
                clipboard::Action::Write(..) => {
                    todo!();
                }
            },
            command::Action::Window(..) => {
                unimplemented!("Use platform specific events instead")
            }
            command::Action::System(action) => match action {
                system::Action::QueryInformation(_tag) => {
                    #[cfg(feature = "system")]
                    {
                        let graphics_info = _graphics_info();
                        let proxy = proxy.clone();

                        let _ = std::thread::spawn(move || {
                            let information =
                                crate::system::information(graphics_info);

                            let message = _tag(information);

                            proxy
                                .send_event(Event::Message(message));
                        });
                    }
                }
            },
            command::Action::Widget(action) => {
                let state = match state {
                    Some(s) => s,
                    None => continue,
                };
                let id = &state.id;

                let mut current_cache = std::mem::take(cache);
                let mut current_operation = Some(Box::new(OperationWrapper::Message(action)));


                let mut user_interface = build_user_interface(
                    application,
                    current_cache,
                    renderer,
                    state.logical_size(),
                    &state.title,
                    debug,
                    id.clone(), // TODO: run the operation on every widget tree ?
                    auto_size_surfaces,
                    proxy
                );

                while let Some(mut operation) = current_operation.take() {
                    user_interface.operate(renderer, operation.as_mut());

                    match operation.as_ref().finish() {
                        operation::Outcome::None => {}
                        operation::Outcome::Some(message) => {
                            match message {
                                operation::OperationOutputWrapper::Message(m) => {
                                    proxy.send_event(Event::SctkEvent(
                                        IcedSctkEvent::UserEvent(m),
                                    ));
                                },
                                operation::OperationOutputWrapper::Id(_) => {
                                    // should not happen
                                },
                            }
                           
                        }
                        operation::Outcome::Chain(mut next) => {
                            current_operation = Some(Box::new(OperationWrapper::Wrapper(next)));
                        }
                    }
                }

                current_cache = user_interface.into_cache();
                *cache = current_cache;
            }
            command::Action::PlatformSpecific(
                platform_specific::Action::Wayland(
                    platform_specific::wayland::Action::LayerSurface(
                        layer_surface_action,
                    ),
                ),
            ) => {
                if let platform_specific::wayland::layer_surface::Action::LayerSurface{ mut builder, _phantom } = layer_surface_action {
                    if builder.size.is_none() {
                        let mut e = application.view(builder.id);
                        let _state = Widget::state(e.as_widget());
                        e.as_widget_mut().diff(&mut Tree::empty());
                        let node = Widget::layout(e.as_widget(), renderer, &builder.size_limits);
                        let bounds = node.bounds();
                        let w = bounds.width.ceil() as u32;
                        let h = bounds.height.ceil() as u32;
                        auto_size_surfaces.insert(SurfaceIdWrapper::LayerSurface(builder.id), (w, h, builder.size_limits, false));
                        builder.size = Some((Some(bounds.width as u32), Some(bounds.height as u32)));
                    }
                    proxy.send_event(Event::LayerSurface(platform_specific::wayland::layer_surface::Action::LayerSurface {builder, _phantom}));
                } else {
                    proxy.send_event(Event::LayerSurface(layer_surface_action));
                }
            }
            command::Action::PlatformSpecific(
                platform_specific::Action::Wayland(
                    platform_specific::wayland::Action::Window(window_action),
                ),
            ) => {
                if let platform_specific::wayland::window::Action::Window{ mut builder, _phantom } = window_action {
                    if builder.autosize {
                        let mut e = application.view(builder.window_id);
                        let _state = Widget::state(e.as_widget());
                        e.as_widget_mut().diff(&mut Tree::empty());
                        let node = Widget::layout(e.as_widget(), renderer, &builder.size_limits);
                        let bounds = node.bounds();
                        let w = bounds.width.ceil() as u32;
                        let h = bounds.height.ceil() as u32;
                        auto_size_surfaces.insert(SurfaceIdWrapper::Window(builder.window_id), (w, h, builder.size_limits, false));
                        builder.size = (bounds.width as u32, bounds.height as u32);
                    }
                    proxy.send_event(Event::Window(platform_specific::wayland::window::Action::Window{builder, _phantom}));
                } else {
                    proxy.send_event(Event::Window(window_action));
                }
            }
            command::Action::PlatformSpecific(
                platform_specific::Action::Wayland(
                    platform_specific::wayland::Action::Popup(popup_action),
                ),
            ) => {
                if let popup::Action::Popup { mut popup, _phantom } = popup_action {
                    if popup.positioner.size.is_none() {
                        let mut e = application.view(popup.id);
                        let _state = Widget::state(e.as_widget());
                        e.as_widget_mut().diff(&mut Tree::empty());
                        let node = Widget::layout(e.as_widget(), renderer, &popup.positioner.size_limits);
                        let bounds = node.bounds();
                        let w = bounds.width.ceil().ceil() as u32;
                        let h = bounds.height.ceil().ceil() as u32;
                        auto_size_surfaces.insert(SurfaceIdWrapper::Popup(popup.id), (w, h, popup.positioner.size_limits, false));
                        popup.positioner.size = Some((bounds.width as u32, bounds.height as u32));
                    }
                    proxy.send_event(Event::Popup(popup::Action::Popup{popup, _phantom}));
                } else {
                    proxy.send_event(Event::Popup(popup_action));
                }
            }
            command::Action::PlatformSpecific(platform_specific::Action::Wayland(platform_specific::wayland::Action::DataDevice(data_device_action))) => {
                proxy.send_event(Event::DataDevice(data_device_action));
            }
            _ => {}
        }
    }
}

pub fn build_user_interfaces<'a, A>(
    application: &'a A,
    renderer: &mut A::Renderer,
    debug: &mut Debug,
    states: &HashMap<SurfaceId, State<A>>,
    mut pure_states: HashMap<SurfaceId, user_interface::Cache>,
    auto_size_surfaces: &mut HashMap<
        SurfaceIdWrapper,
        (u32, u32, Limits, bool),
    >,
    ev_proxy: &mut proxy::Proxy<Event<A::Message>>,
) -> HashMap<
    SurfaceId,
    UserInterface<
        'a,
        <A as Program>::Message,
        <A as Program>::Renderer,
    >,
>
where
    A: Application + 'static,
    <A::Renderer as Renderer>::Theme: StyleSheet,
{
    let mut interfaces = HashMap::new();

    // TODO ASHLEY make sure Ids are iterated in the same order every time for a11y
    for (id, pure_state) in pure_states.drain().sorted_by(|a, b| a.0.cmp(&b.0))
    {
        let state = &states.get(&id).unwrap();

        let user_interface = build_user_interface(
            application,
            pure_state,
            renderer,
            state.logical_size(),
            &state.title,
            debug,
            state.id,
            auto_size_surfaces,
            ev_proxy,
        );

        let _ = interfaces.insert(id, user_interface);
    }

    interfaces
}

// Determine if `SctkEvent` is for surface with given object id.
fn event_is_for_surface(
    evt: &SctkEvent,
    object_id: &ObjectId,
    has_kbd_focus: bool,
) -> bool {
    match evt {
        SctkEvent::SeatEvent { id, .. } => &id.id() == object_id,
        SctkEvent::PointerEvent { variant, .. } => {
            &variant.surface.id() == object_id
        }
        SctkEvent::KeyboardEvent { variant, .. } => match variant {
            KeyboardEventVariant::Leave(id) => &id.id() == object_id,
            _ => has_kbd_focus,
        },
        SctkEvent::WindowEvent { id, .. } => &id.id() == object_id,
        SctkEvent::LayerSurfaceEvent { id, .. } => &id.id() == object_id,
        SctkEvent::PopupEvent { id, .. } => &id.id() == object_id,
        SctkEvent::Frame(_)
        | SctkEvent::NewOutput { .. }
        | SctkEvent::UpdateOutput { .. }
        | SctkEvent::RemovedOutput(_) => false,
        SctkEvent::ScaleFactorChanged { id, .. } => &id.id() == object_id,
        SctkEvent::DndOffer { surface, .. } => &surface.id() == object_id,
        SctkEvent::SelectionOffer(_) => true,
        SctkEvent::DataSource(_) => true,
    }
}
