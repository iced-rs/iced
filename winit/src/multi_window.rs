//! Create interactive, native cross-platform applications for WGPU.
mod state;
mod windows;

pub use state::State;

use crate::conversion;
use crate::core::widget::operation;
use crate::core::{self, mouse, renderer, window, Size};
use crate::futures::futures::channel::mpsc;
use crate::futures::futures::{task, Future, FutureExt, StreamExt};
use crate::futures::{Executor, Runtime, Subscription};
use crate::graphics::{compositor, Compositor};
use crate::multi_window::windows::Windows;
use crate::runtime::command::{self, Command};
use crate::runtime::multi_window::Program;
use crate::runtime::user_interface::{self, UserInterface};
use crate::runtime::Debug;
use crate::style::application::StyleSheet;
use crate::{Clipboard, Error, Proxy, Settings};

use std::mem::ManuallyDrop;
use std::time::Instant;
use winit::monitor::MonitorHandle;

/// This is a wrapper around the `Application::Message` associate type
/// to allows the `shell` to create internal messages, while still having
/// the current user-specified custom messages.
#[derive(Debug)]
pub enum Event<Message> {
    /// An internal event which contains an [`Application`] generated message.
    Application(Message),
    /// An internal event which spawns a new window.
    NewWindow {
        /// The [window::Id] of the newly spawned [`Window`].
        id: window::Id,
        /// The [settings::Window] of the newly spawned [`Window`].
        settings: window::Settings,
        /// The title of the newly spawned [`Window`].
        title: String,
        /// The monitor on which to spawn the window. If `None`, will use monitor of the last window
        /// spawned.
        monitor: Option<MonitorHandle>,
    },
    /// An internal event for closing a window.
    CloseWindow(window::Id),
    /// An internal event for when the window has finished being created.
    WindowCreated {
        /// The internal ID of the window.
        id: window::Id,
        /// The raw window.
        window: winit::window::Window,
        /// Whether or not the window should close when a user requests it does.
        exit_on_close_request: bool,
    },
}

#[allow(unsafe_code)]
unsafe impl<Message> std::marker::Send for Event<Message> {}

/// An interactive, native, cross-platform, multi-windowed application.
///
/// This trait is the main entrypoint of multi-window Iced. Once implemented, you can run
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
    <Self::Renderer as core::Renderer>::Theme: StyleSheet,
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
    fn title(&self, window: window::Id) -> String;

    /// Returns the current `Theme` of the [`Application`].
    fn theme(
        &self,
        window: window::Id,
    ) -> <Self::Renderer as core::Renderer>::Theme;

    /// Returns the `Style` variation of the `Theme`.
    fn style(
        &self,
    ) -> <<Self::Renderer as core::Renderer>::Theme as StyleSheet>::Style {
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

    /// Returns the scale factor of the window of the [`Application`].
    ///
    /// It can be used to dynamically control the size of the UI at runtime
    /// (i.e. zooming).
    ///
    /// For instance, a scale factor of `2.0` will make widgets twice as big,
    /// while a scale factor of `0.5` will shrink them to half their size.
    ///
    /// By default, it returns `1.0`.
    #[allow(unused_variables)]
    fn scale_factor(&self, window: window::Id) -> f64 {
        1.0
    }
}

/// Runs an [`Application`] with an executor, compositor, and the provided
/// settings.
pub fn run<A, E, C>(
    settings: Settings<A::Flags>,
    compositor_settings: C::Settings,
) -> Result<(), Error>
where
    A: Application + 'static,
    E: Executor + 'static,
    C: Compositor<Renderer = A::Renderer> + 'static,
    <A::Renderer as core::Renderer>::Theme: StyleSheet,
{
    use winit::event_loop::EventLoopBuilder;

    let mut debug = Debug::new();
    debug.startup_started();

    let event_loop = EventLoopBuilder::with_user_event().build();
    let proxy = event_loop.create_proxy();

    let runtime = {
        let proxy = Proxy::new(event_loop.create_proxy());
        let executor = E::new().map_err(Error::ExecutorCreationFailed)?;

        Runtime::new(executor, proxy)
    };

    let (application, init_command) = {
        let flags = settings.flags;

        runtime.enter(|| A::new(flags))
    };

    let should_main_be_visible = settings.window.visible;
    let exit_on_close_request = settings.window.exit_on_close_request;

    let builder = conversion::window_settings(
        settings.window,
        &application.title(window::Id::MAIN),
        event_loop.primary_monitor(),
        settings.id,
    )
    .with_visible(false);

    log::info!("Window builder: {:#?}", builder);

    let main_window = builder
        .build(&event_loop)
        .map_err(Error::WindowCreationFailed)?;

    #[cfg(target_arch = "wasm32")]
    {
        use winit::platform::web::WindowExtWebSys;

        let canvas = main_window.canvas();

        let window = web_sys::window().unwrap();
        let document = window.document().unwrap();
        let body = document.body().unwrap();

        let target = target.and_then(|target| {
            body.query_selector(&format!("#{}", target))
                .ok()
                .unwrap_or(None)
        });

        match target {
            Some(node) => {
                let _ = node
                    .replace_with_with_node_1(&canvas)
                    .expect(&format!("Could not replace #{}", node.id()));
            }
            None => {
                let _ = body
                    .append_child(&canvas)
                    .expect("Append canvas to HTML body");
            }
        };
    }

    let (mut compositor, renderer) =
        C::new(compositor_settings, Some(&main_window))?;

    let windows = Windows::new(
        &application,
        &mut compositor,
        renderer,
        main_window,
        exit_on_close_request,
    );

    let (mut event_sender, event_receiver) = mpsc::unbounded();
    let (control_sender, mut control_receiver) = mpsc::unbounded();

    let mut instance = Box::pin(run_instance::<A, E, C>(
        application,
        compositor,
        runtime,
        proxy,
        debug,
        event_receiver,
        control_sender,
        init_command,
        windows,
        should_main_be_visible,
    ));

    let mut context = task::Context::from_waker(task::noop_waker_ref());

    platform::run(event_loop, move |event, window_target, control_flow| {
        use winit::event_loop::ControlFlow;

        if let ControlFlow::ExitWithCode(_) = control_flow {
            return;
        }

        let event = match event {
            winit::event::Event::WindowEvent {
                event:
                    winit::event::WindowEvent::ScaleFactorChanged {
                        new_inner_size,
                        ..
                    },
                window_id,
            } => Some(winit::event::Event::WindowEvent {
                event: winit::event::WindowEvent::Resized(*new_inner_size),
                window_id,
            }),
            winit::event::Event::UserEvent(Event::NewWindow {
                id,
                settings,
                title,
                monitor,
            }) => {
                let exit_on_close_request = settings.exit_on_close_request;

                let window = conversion::window_settings(
                    settings, &title, monitor, None,
                )
                .build(window_target)
                .expect("Failed to build window");

                Some(winit::event::Event::UserEvent(Event::WindowCreated {
                    id,
                    window,
                    exit_on_close_request,
                }))
            }
            _ => event.to_static(),
        };

        if let Some(event) = event {
            event_sender.start_send(event).expect("Send event");

            let poll = instance.as_mut().poll(&mut context);

            match poll {
                task::Poll::Pending => {
                    if let Ok(Some(flow)) = control_receiver.try_next() {
                        *control_flow = flow;
                    }
                }
                task::Poll::Ready(_) => {
                    *control_flow = ControlFlow::Exit;
                }
            };
        }
    })
}

async fn run_instance<A, E, C>(
    mut application: A,
    mut compositor: C,
    mut runtime: Runtime<E, Proxy<Event<A::Message>>, Event<A::Message>>,
    mut proxy: winit::event_loop::EventLoopProxy<Event<A::Message>>,
    mut debug: Debug,
    mut event_receiver: mpsc::UnboundedReceiver<
        winit::event::Event<'_, Event<A::Message>>,
    >,
    mut control_sender: mpsc::UnboundedSender<winit::event_loop::ControlFlow>,
    init_command: Command<A::Message>,
    mut windows: Windows<A, C>,
    should_main_window_be_visible: bool,
) where
    A: Application + 'static,
    E: Executor + 'static,
    C: Compositor<Renderer = A::Renderer> + 'static,
    <A::Renderer as core::Renderer>::Theme: StyleSheet,
{
    use winit::event;
    use winit::event_loop::ControlFlow;

    let mut clipboard = Clipboard::connect(windows.main());

    let mut ui_caches = vec![user_interface::Cache::default()];
    let mut user_interfaces = ManuallyDrop::new(build_user_interfaces(
        &application,
        &mut debug,
        &mut windows,
        vec![user_interface::Cache::default()],
    ));

    if should_main_window_be_visible {
        windows.main().set_visible(true);
    }

    run_command(
        &application,
        &mut compositor,
        init_command,
        &mut runtime,
        &mut clipboard,
        &mut proxy,
        &mut debug,
        &mut windows,
        &mut ui_caches,
    );

    runtime.track(
        application
            .subscription()
            .map(Event::Application)
            .into_recipes(),
    );

    let mut mouse_interaction = mouse::Interaction::default();

    let mut events =
        if let Some((position, size)) = logical_bounds_of(windows.main()) {
            vec![(
                Some(window::Id::MAIN),
                core::Event::Window(
                    window::Id::MAIN,
                    window::Event::Created { position, size },
                ),
            )]
        } else {
            Vec::new()
        };
    let mut messages = Vec::new();
    let mut redraw_pending = false;

    debug.startup_finished();

    'main: while let Some(event) = event_receiver.next().await {
        match event {
            event::Event::NewEvents(start_cause) => {
                redraw_pending = matches!(
                    start_cause,
                    event::StartCause::Init
                        | event::StartCause::Poll
                        | event::StartCause::ResumeTimeReached { .. }
                );
            }
            event::Event::MainEventsCleared => {
                debug.event_processing_started();
                let mut uis_stale = false;

                for (i, id) in windows.ids.iter().enumerate() {
                    let mut window_events = vec![];

                    events.retain(|(window_id, event)| {
                        if *window_id == Some(*id) || window_id.is_none() {
                            window_events.push(event.clone());
                            false
                        } else {
                            true
                        }
                    });

                    if !redraw_pending
                        && window_events.is_empty()
                        && messages.is_empty()
                    {
                        continue;
                    }

                    let (ui_state, statuses) = user_interfaces[i].update(
                        &window_events,
                        windows.states[i].cursor(),
                        &mut windows.renderers[i],
                        &mut clipboard,
                        &mut messages,
                    );

                    if !uis_stale {
                        uis_stale =
                            matches!(ui_state, user_interface::State::Outdated);
                    }

                    for (event, status) in
                        window_events.into_iter().zip(statuses.into_iter())
                    {
                        runtime.broadcast(event, status);
                    }
                }

                debug.event_processing_finished();

                // TODO mw application update returns which window IDs to update
                if !messages.is_empty() || uis_stale {
                    let mut cached_interfaces: Vec<user_interface::Cache> =
                        ManuallyDrop::into_inner(user_interfaces)
                            .drain(..)
                            .map(UserInterface::into_cache)
                            .collect();

                    // Update application
                    update(
                        &mut application,
                        &mut compositor,
                        &mut runtime,
                        &mut clipboard,
                        &mut proxy,
                        &mut debug,
                        &mut messages,
                        &mut windows,
                        &mut cached_interfaces,
                    );

                    // we must synchronize all window states with application state after an
                    // application update since we don't know what changed
                    for (state, (id, window)) in windows
                        .states
                        .iter_mut()
                        .zip(windows.ids.iter().zip(windows.raw.iter()))
                    {
                        state.synchronize(&application, *id, window);
                    }

                    // rebuild UIs with the synchronized states
                    user_interfaces = ManuallyDrop::new(build_user_interfaces(
                        &application,
                        &mut debug,
                        &mut windows,
                        cached_interfaces,
                    ));
                }

                debug.draw_started();

                for (i, id) in windows.ids.iter().enumerate() {
                    // TODO: Avoid redrawing all the time by forcing widgets to
                    //  request redraws on state changes
                    //
                    // Then, we can use the `interface_state` here to decide if a redraw
                    // is needed right away, or simply wait until a specific time.
                    let redraw_event = core::Event::Window(
                        *id,
                        window::Event::RedrawRequested(Instant::now()),
                    );

                    let cursor = windows.states[i].cursor();

                    let (ui_state, _) = user_interfaces[i].update(
                        &[redraw_event.clone()],
                        cursor,
                        &mut windows.renderers[i],
                        &mut clipboard,
                        &mut messages,
                    );

                    let new_mouse_interaction = {
                        let state = &windows.states[i];

                        user_interfaces[i].draw(
                            &mut windows.renderers[i],
                            state.theme(),
                            &renderer::Style {
                                text_color: state.text_color(),
                            },
                            cursor,
                        )
                    };

                    if new_mouse_interaction != mouse_interaction {
                        windows.raw[i].set_cursor_icon(
                            conversion::mouse_interaction(
                                new_mouse_interaction,
                            ),
                        );

                        mouse_interaction = new_mouse_interaction;
                    }

                    // TODO once widgets can request to be redrawn, we can avoid always requesting a
                    // redraw
                    windows.raw[i].request_redraw();

                    runtime.broadcast(
                        redraw_event.clone(),
                        core::event::Status::Ignored,
                    );

                    let _ = control_sender.start_send(match ui_state {
                        user_interface::State::Updated {
                            redraw_request: Some(redraw_request),
                        } => match redraw_request {
                            window::RedrawRequest::NextFrame => {
                                ControlFlow::Poll
                            }
                            window::RedrawRequest::At(at) => {
                                ControlFlow::WaitUntil(at)
                            }
                        },
                        _ => ControlFlow::Wait,
                    });
                }

                redraw_pending = false;

                debug.draw_finished();
            }
            event::Event::PlatformSpecific(event::PlatformSpecific::MacOS(
                event::MacOS::ReceivedUrl(url),
            )) => {
                use crate::core::event;

                events.push((
                    None,
                    event::Event::PlatformSpecific(
                        event::PlatformSpecific::MacOS(
                            event::MacOS::ReceivedUrl(url),
                        ),
                    ),
                ));
            }
            event::Event::UserEvent(event) => match event {
                Event::Application(message) => {
                    messages.push(message);
                }
                Event::WindowCreated {
                    id,
                    window,
                    exit_on_close_request,
                } => {
                    let bounds = logical_bounds_of(&window);

                    let (inner_size, i) = windows.add(
                        &application,
                        &mut compositor,
                        id,
                        window,
                        exit_on_close_request,
                    );

                    user_interfaces.push(build_user_interface(
                        &application,
                        user_interface::Cache::default(),
                        &mut windows.renderers[i],
                        inner_size,
                        &mut debug,
                        id,
                    ));
                    ui_caches.push(user_interface::Cache::default());

                    if let Some(bounds) = bounds {
                        events.push((
                            Some(id),
                            core::Event::Window(
                                id,
                                window::Event::Created {
                                    position: bounds.0,
                                    size: bounds.1,
                                },
                            ),
                        ));
                    }
                }
                Event::CloseWindow(id) => {
                    let i = windows.delete(id);
                    let _ = user_interfaces.remove(i);
                    let _ = ui_caches.remove(i);

                    if windows.is_empty() {
                        break 'main;
                    }
                }
                Event::NewWindow { .. } => unreachable!(),
            },
            event::Event::RedrawRequested(id) => {
                let i = windows.index_from_raw(id);
                let state = &windows.states[i];
                let physical_size = state.physical_size();

                if physical_size.width == 0 || physical_size.height == 0 {
                    continue;
                }

                debug.render_started();
                let current_viewport_version = state.viewport_version();
                let window_viewport_version = windows.viewport_versions[i];

                if window_viewport_version != current_viewport_version {
                    let logical_size = state.logical_size();

                    debug.layout_started();

                    let renderer = &mut windows.renderers[i];
                    let ui = user_interfaces.remove(i);

                    user_interfaces
                        .insert(i, ui.relayout(logical_size, renderer));

                    debug.layout_finished();

                    debug.draw_started();
                    let new_mouse_interaction = user_interfaces[i].draw(
                        renderer,
                        state.theme(),
                        &renderer::Style {
                            text_color: state.text_color(),
                        },
                        state.cursor(),
                    );

                    if new_mouse_interaction != mouse_interaction {
                        windows.raw[i].set_cursor_icon(
                            conversion::mouse_interaction(
                                new_mouse_interaction,
                            ),
                        );

                        mouse_interaction = new_mouse_interaction;
                    }
                    debug.draw_finished();

                    compositor.configure_surface(
                        &mut windows.surfaces[i],
                        physical_size.width,
                        physical_size.height,
                    );

                    windows.viewport_versions[i] = current_viewport_version;
                }

                match compositor.present(
                    &mut windows.renderers[i],
                    &mut windows.surfaces[i],
                    state.viewport(),
                    state.background_color(),
                    &debug.overlay(),
                ) {
                    Ok(()) => {
                        debug.render_finished();

                        // TODO: Handle animations!
                        // Maybe we can use `ControlFlow::WaitUntil` for this.
                    }
                    Err(error) => match error {
                        // This is an unrecoverable error.
                        compositor::SurfaceError::OutOfMemory => {
                            panic!("{:?}", error);
                        }
                        _ => {
                            debug.render_finished();
                            log::error!(
                                "Error {error:?} when presenting surface."
                            );

                            // Try rendering all windows again next frame.
                            for window in &windows.raw {
                                window.request_redraw();
                            }
                        }
                    },
                }
            }
            event::Event::WindowEvent {
                event: window_event,
                window_id,
            } => {
                let window_index =
                    windows.raw.iter().position(|w| w.id() == window_id);

                match window_index {
                    Some(i) => {
                        let id = windows.ids[i];
                        let raw = &windows.raw[i];
                        let exit_on_close_request =
                            windows.exit_on_close_requested[i];

                        if matches!(
                            window_event,
                            winit::event::WindowEvent::CloseRequested
                        ) && exit_on_close_request
                        {
                            let i = windows.delete(id);
                            let _ = user_interfaces.remove(i);
                            let _ = ui_caches.remove(i);

                            if windows.is_empty() {
                                break 'main;
                            }
                        } else {
                            let state = &mut windows.states[i];
                            state.update(raw, &window_event, &mut debug);

                            if let Some(event) = conversion::window_event(
                                id,
                                &window_event,
                                state.scale_factor(),
                                state.modifiers(),
                            ) {
                                events.push((Some(id), event));
                            }
                        }
                    }
                    None => {
                        // This is the only special case, since in order to trigger the Destroyed event the
                        // window reference from winit must be dropped, but we still want to inform the
                        // user that the window was destroyed so they can clean up any specific window
                        // code for this window
                        if matches!(
                            window_event,
                            winit::event::WindowEvent::Destroyed
                        ) {
                            let id = windows.get_pending_destroy(window_id);

                            events.push((
                                None,
                                core::Event::Window(
                                    id,
                                    window::Event::Destroyed,
                                ),
                            ));
                        }
                    }
                }
            }
            _ => {}
        }
    }

    let _ = ManuallyDrop::into_inner(user_interfaces);
}

/// Builds a window's [`UserInterface`] for the [`Application`].
pub fn build_user_interface<'a, A: Application>(
    application: &'a A,
    cache: user_interface::Cache,
    renderer: &mut A::Renderer,
    size: Size,
    debug: &mut Debug,
    id: window::Id,
) -> UserInterface<'a, A::Message, A::Renderer>
where
    <A::Renderer as core::Renderer>::Theme: StyleSheet,
{
    debug.view_started();
    let view = application.view(id);
    debug.view_finished();

    debug.layout_started();
    let user_interface = UserInterface::build(view, size, cache, renderer);
    debug.layout_finished();

    user_interface
}

/// Updates a multi-window [`Application`] by feeding it messages, spawning any
/// resulting [`Command`], and tracking its [`Subscription`].
pub fn update<A: Application, C, E: Executor>(
    application: &mut A,
    compositor: &mut C,
    runtime: &mut Runtime<E, Proxy<Event<A::Message>>, Event<A::Message>>,
    clipboard: &mut Clipboard,
    proxy: &mut winit::event_loop::EventLoopProxy<Event<A::Message>>,
    debug: &mut Debug,
    messages: &mut Vec<A::Message>,
    windows: &mut Windows<A, C>,
    ui_caches: &mut Vec<user_interface::Cache>,
) where
    C: Compositor<Renderer = A::Renderer> + 'static,
    <A::Renderer as core::Renderer>::Theme: StyleSheet,
{
    for message in messages.drain(..) {
        debug.log_message(&message);
        debug.update_started();

        let command = runtime.enter(|| application.update(message));
        debug.update_finished();

        run_command(
            application,
            compositor,
            command,
            runtime,
            clipboard,
            proxy,
            debug,
            windows,
            ui_caches,
        );
    }

    let subscription = application.subscription().map(Event::Application);
    runtime.track(subscription.into_recipes());
}

/// Runs the actions of a [`Command`].
pub fn run_command<A, C, E>(
    application: &A,
    compositor: &mut C,
    command: Command<A::Message>,
    runtime: &mut Runtime<E, Proxy<Event<A::Message>>, Event<A::Message>>,
    clipboard: &mut Clipboard,
    proxy: &mut winit::event_loop::EventLoopProxy<Event<A::Message>>,
    debug: &mut Debug,
    windows: &mut Windows<A, C>,
    ui_caches: &mut Vec<user_interface::Cache>,
) where
    A: Application,
    E: Executor,
    C: Compositor<Renderer = A::Renderer> + 'static,
    <A::Renderer as core::Renderer>::Theme: StyleSheet,
{
    use crate::runtime::clipboard;
    use crate::runtime::system;
    use crate::runtime::window;

    for action in command.actions() {
        match action {
            command::Action::Future(future) => {
                runtime.spawn(Box::pin(future.map(Event::Application)));
            }
            command::Action::Stream(stream) => {
                runtime.run(Box::pin(stream.map(Event::Application)));
            }
            command::Action::Clipboard(action) => match action {
                clipboard::Action::Read(tag) => {
                    let message = tag(clipboard.read());

                    proxy
                        .send_event(Event::Application(message))
                        .expect("Send message to event loop");
                }
                clipboard::Action::Write(contents) => {
                    clipboard.write(contents);
                }
            },
            command::Action::Window(id, action) => match action {
                window::Action::Spawn { settings } => {
                    let monitor = windows.last_monitor();

                    proxy
                        .send_event(Event::NewWindow {
                            id,
                            settings,
                            title: application.title(id),
                            monitor,
                        })
                        .expect("Send message to event loop");
                }
                window::Action::Close => {
                    proxy
                        .send_event(Event::CloseWindow(id))
                        .expect("Send message to event loop");
                }
                window::Action::Drag => {
                    let _ = windows.with_raw(id).drag_window();
                }
                window::Action::Resize(size) => {
                    windows.with_raw(id).set_inner_size(
                        winit::dpi::LogicalSize {
                            width: size.width,
                            height: size.height,
                        },
                    );
                }
                window::Action::FetchSize(callback) => {
                    let window = windows.with_raw(id);
                    let size = window.inner_size();

                    proxy
                        .send_event(Event::Application(callback(Size::new(
                            size.width,
                            size.height,
                        ))))
                        .expect("Send message to event loop")
                }
                window::Action::Maximize(maximized) => {
                    windows.with_raw(id).set_maximized(maximized);
                }
                window::Action::Minimize(minimized) => {
                    windows.with_raw(id).set_minimized(minimized);
                }
                window::Action::Move { x, y } => {
                    windows.with_raw(id).set_outer_position(
                        winit::dpi::LogicalPosition { x, y },
                    );
                }
                window::Action::ChangeMode(mode) => {
                    let window = windows.with_raw(id);
                    window.set_visible(conversion::visible(mode));
                    window.set_fullscreen(conversion::fullscreen(
                        window.current_monitor(),
                        mode,
                    ));
                }
                window::Action::ChangeIcon(icon) => {
                    windows.with_raw(id).set_window_icon(conversion::icon(icon))
                }
                window::Action::FetchMode(tag) => {
                    let window = windows.with_raw(id);
                    let mode = if window.is_visible().unwrap_or(true) {
                        conversion::mode(window.fullscreen())
                    } else {
                        core::window::Mode::Hidden
                    };

                    proxy
                        .send_event(Event::Application(tag(mode)))
                        .expect("Event loop doesn't exist.");
                }
                window::Action::ToggleMaximize => {
                    let window = windows.with_raw(id);
                    window.set_maximized(!window.is_maximized());
                }
                window::Action::ToggleDecorations => {
                    let window = windows.with_raw(id);
                    window.set_decorations(!window.is_decorated());
                }
                window::Action::RequestUserAttention(attention_type) => {
                    windows.with_raw(id).request_user_attention(
                        attention_type.map(conversion::user_attention),
                    );
                }
                window::Action::GainFocus => {
                    windows.with_raw(id).focus_window();
                }
                window::Action::ChangeLevel(level) => {
                    windows
                        .with_raw(id)
                        .set_window_level(conversion::window_level(level));
                }
                window::Action::FetchId(tag) => proxy
                    .send_event(Event::Application(tag(windows
                        .with_raw(id)
                        .id()
                        .into())))
                    .expect("Event loop doesn't exist."),
                window::Action::Screenshot(tag) => {
                    let i = windows.index_from_id(id);
                    let state = &windows.states[i];
                    let surface = &mut windows.surfaces[i];
                    let renderer = &mut windows.renderers[i];

                    let bytes = compositor.screenshot(
                        renderer,
                        surface,
                        state.viewport(),
                        state.background_color(),
                        &debug.overlay(),
                    );

                    proxy
                        .send_event(Event::Application(tag(
                            window::Screenshot::new(
                                bytes,
                                state.physical_size(),
                            ),
                        )))
                        .expect("Event loop doesn't exist.")
                }
            },
            command::Action::System(action) => match action {
                system::Action::QueryInformation(_tag) => {
                    #[cfg(feature = "system")]
                    {
                        let graphics_info = compositor.fetch_information();
                        let proxy = proxy.clone();

                        let _ = std::thread::spawn(move || {
                            let information =
                                crate::system::information(graphics_info);

                            let message = _tag(information);

                            proxy
                                .send_event(Event::Application(message))
                                .expect("Event loop doesn't exist.")
                        });
                    }
                }
            },
            command::Action::Widget(action) => {
                let mut current_operation = Some(action);

                let mut uis = build_user_interfaces(
                    application,
                    debug,
                    windows,
                    std::mem::take(ui_caches),
                );

                'operate: while let Some(mut operation) =
                    current_operation.take()
                {
                    for (i, ui) in uis.iter_mut().enumerate() {
                        ui.operate(&windows.renderers[i], operation.as_mut());

                        match operation.finish() {
                            operation::Outcome::None => {}
                            operation::Outcome::Some(message) => {
                                proxy
                                    .send_event(Event::Application(message))
                                    .expect("Event loop doesn't exist.");

                                // operation completed, don't need to try to operate on rest of UIs
                                break 'operate;
                            }
                            operation::Outcome::Chain(next) => {
                                current_operation = Some(next);
                            }
                        }
                    }
                }

                *ui_caches =
                    uis.drain(..).map(UserInterface::into_cache).collect();
            }
            command::Action::LoadFont { bytes, tagger } => {
                use crate::core::text::Renderer;

                // TODO change this once we change each renderer to having a single backend reference.. :pain:
                // TODO: Error handling (?)
                for renderer in &mut windows.renderers {
                    renderer.load_font(bytes.clone());
                }

                proxy
                    .send_event(Event::Application(tagger(Ok(()))))
                    .expect("Send message to event loop");
            }
        }
    }
}

/// Build the user interface for every window.
pub fn build_user_interfaces<'a, A: Application, C: Compositor>(
    application: &'a A,
    debug: &mut Debug,
    windows: &mut Windows<A, C>,
    mut cached_user_interfaces: Vec<user_interface::Cache>,
) -> Vec<UserInterface<'a, A::Message, A::Renderer>>
where
    <A::Renderer as core::Renderer>::Theme: StyleSheet,
    C: Compositor<Renderer = A::Renderer>,
{
    cached_user_interfaces
        .drain(..)
        .zip(
            windows
                .ids
                .iter()
                .zip(windows.states.iter().zip(windows.renderers.iter_mut())),
        )
        .fold(vec![], |mut uis, (cache, (id, (state, renderer)))| {
            uis.push(build_user_interface(
                application,
                cache,
                renderer,
                state.logical_size(),
                debug,
                *id,
            ));

            uis
        })
}

/// Returns true if the provided event should cause an [`Application`] to
/// exit.
pub fn user_force_quit(
    event: &winit::event::WindowEvent<'_>,
    _modifiers: winit::event::ModifiersState,
) -> bool {
    match event {
        #[cfg(target_os = "macos")]
        winit::event::WindowEvent::KeyboardInput {
            input:
                winit::event::KeyboardInput {
                    virtual_keycode: Some(winit::event::VirtualKeyCode::Q),
                    state: winit::event::ElementState::Pressed,
                    ..
                },
            ..
        } if _modifiers.logo() => true,
        _ => false,
    }
}

fn logical_bounds_of(
    window: &winit::window::Window,
) -> Option<((i32, i32), Size<u32>)> {
    let scale = window.scale_factor();
    let pos = window
        .inner_position()
        .map(|pos| {
            ((pos.x as f64 / scale) as i32, (pos.y as f64 / scale) as i32)
        })
        .ok()?;
    let size = {
        let size = window.inner_size();
        Size::new(
            (size.width as f64 / scale) as u32,
            (size.height as f64 / scale) as u32,
        )
    };

    Some((pos, size))
}

#[cfg(not(target_arch = "wasm32"))]
mod platform {
    pub fn run<T, F>(
        mut event_loop: winit::event_loop::EventLoop<T>,
        event_handler: F,
    ) -> Result<(), super::Error>
    where
        F: 'static
            + FnMut(
                winit::event::Event<'_, T>,
                &winit::event_loop::EventLoopWindowTarget<T>,
                &mut winit::event_loop::ControlFlow,
            ),
    {
        use winit::platform::run_return::EventLoopExtRunReturn;

        let _ = event_loop.run_return(event_handler);

        Ok(())
    }
}

#[cfg(target_arch = "wasm32")]
mod platform {
    pub fn run<T, F>(
        event_loop: winit::event_loop::EventLoop<T>,
        event_handler: F,
    ) -> !
    where
        F: 'static
            + FnMut(
                winit::event::Event<'_, T>,
                &winit::event_loop::EventLoopWindowTarget<T>,
                &mut winit::event_loop::ControlFlow,
            ),
    {
        event_loop.run(event_handler)
    }
}
