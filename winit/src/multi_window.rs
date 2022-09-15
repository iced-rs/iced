//! Create interactive, native cross-platform applications.
mod state;

pub use state::State;

use crate::clipboard::{self, Clipboard};
use crate::conversion;
use crate::mouse;
use crate::renderer;
use crate::settings;
use crate::widget::operation;
use crate::window;
use crate::{
    Command, Debug, Element, Error, Executor, Proxy, Renderer, Runtime,
    Settings, Size, Subscription,
};

use iced_futures::futures::channel::mpsc;
use iced_futures::futures::{self, FutureExt};
use iced_graphics::compositor;
use iced_native::user_interface::{self, UserInterface};

pub use iced_native::application::{Appearance, StyleSheet};

use std::collections::HashMap;
use std::mem::ManuallyDrop;

/// TODO(derezzedex)
// This is the an wrapper around the `Application::Message` associate type
// to allows the `shell` to create internal messages, while still having
// the current user specified custom messages.
#[derive(Debug)]
pub enum Event<Message> {
    /// An [`Application`] generated message
    Application(Message),

    /// TODO(derezzedex)
    // Create a wrapper variant of `window::Event` type instead
    // (maybe we should also allow users to listen/react to those internal messages?)
    NewWindow(window::Id, settings::Window),
    /// TODO(derezzedex)
    CloseWindow(window::Id),
    /// TODO(derezzedex)
    WindowCreated(window::Id, winit::window::Window),
}

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
pub trait Application: Sized
where
    <Self::Renderer as crate::Renderer>::Theme: StyleSheet,
{
    /// The data needed to initialize your [`Application`].
    type Flags;

    /// The graphics backend to use to draw the [`Program`].
    type Renderer: Renderer;

    /// The type of __messages__ your [`Program`] will produce.
    type Message: std::fmt::Debug + Send;

    /// TODO(derezzedex)
    fn windows(&self) -> Vec<(window::Id, settings::Window)>;

    /// Handles a __message__ and updates the state of the [`Program`].
    ///
    /// This is where you define your __update logic__. All the __messages__,
    /// produced by either user interactions or commands, will be handled by
    /// this method.
    ///
    /// Any [`Command`] returned will be executed immediately in the
    /// background by shells.
    fn update(&mut self, message: Self::Message) -> Command<Self::Message>;

    /// Returns the widgets to display in the [`Program`].
    ///
    /// These widgets can produce __messages__ based on user interaction.
    fn view(
        &self,
        window: window::Id,
    ) -> Element<'_, Self::Message, Self::Renderer>;

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
    fn theme(&self) -> <Self::Renderer as crate::Renderer>::Theme;

    /// Returns the [`Style`] variation of the [`Theme`].
    fn style(
        &self,
    ) -> <<Self::Renderer as crate::Renderer>::Theme as StyleSheet>::Style {
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

    /// TODO(derezzedex)
    fn close_requested(&self, window: window::Id) -> Self::Message;
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
    C: iced_graphics::window::Compositor<Renderer = A::Renderer> + 'static,
    <A::Renderer as crate::Renderer>::Theme: StyleSheet,
{
    use futures::task;
    use futures::Future;
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

    let builder = settings.window.into_builder(
        &application.title(),
        event_loop.primary_monitor(),
        settings.id,
    );

    log::info!("Window builder: {:#?}", builder);

    let window = builder
        .build(&event_loop)
        .map_err(Error::WindowCreationFailed)?;

    let windows: HashMap<window::Id, winit::window::Window> =
        HashMap::from([(window::Id::new(0usize), window)]);
    let window = windows.values().next().expect("No window found");

    #[cfg(target_arch = "wasm32")]
    {
        use winit::platform::web::WindowExtWebSys;

        let canvas = window.canvas();

        let window = web_sys::window().unwrap();
        let document = window.document().unwrap();
        let body = document.body().unwrap();

        let _ = body
            .append_child(&canvas)
            .expect("Append canvas to HTML body");
    }

    let (compositor, renderer) = C::new(compositor_settings, Some(&window))?;

    let (mut sender, receiver) = mpsc::unbounded();

    let mut instance = Box::pin(run_instance::<A, E, C>(
        application,
        compositor,
        renderer,
        runtime,
        proxy,
        debug,
        receiver,
        init_command,
        windows,
        settings.exit_on_close_request,
    ));

    let mut context = task::Context::from_waker(task::noop_waker_ref());

    platform::run(event_loop, move |event, event_loop, control_flow| {
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
            winit::event::Event::UserEvent(Event::NewWindow(id, settings)) => {
                // TODO(derezzedex)
                let window = settings
                    .into_builder(
                        "fix window title",
                        event_loop.primary_monitor(),
                        None,
                    )
                    .build(event_loop)
                    .expect("Failed to build window");

                Some(winit::event::Event::UserEvent(Event::WindowCreated(
                    id, window,
                )))
            }
            _ => event.to_static(),
        };

        if let Some(event) = event {
            sender.start_send(event).expect("Send event");

            let poll = instance.as_mut().poll(&mut context);

            *control_flow = match poll {
                task::Poll::Pending => ControlFlow::Wait,
                task::Poll::Ready(_) => ControlFlow::Exit,
            };
        }
    })
}

async fn run_instance<A, E, C>(
    mut application: A,
    mut compositor: C,
    mut renderer: A::Renderer,
    mut runtime: Runtime<E, Proxy<Event<A::Message>>, Event<A::Message>>,
    mut proxy: winit::event_loop::EventLoopProxy<Event<A::Message>>,
    mut debug: Debug,
    mut receiver: mpsc::UnboundedReceiver<
        winit::event::Event<'_, Event<A::Message>>,
    >,
    init_command: Command<A::Message>,
    mut windows: HashMap<window::Id, winit::window::Window>,
    _exit_on_close_request: bool,
) where
    A: Application + 'static,
    E: Executor + 'static,
    C: iced_graphics::window::Compositor<Renderer = A::Renderer> + 'static,
    <A::Renderer as crate::Renderer>::Theme: StyleSheet,
{
    use iced_futures::futures::stream::StreamExt;
    use winit::event;

    let mut clipboard =
        Clipboard::connect(windows.values().next().expect("No window found"));
    let mut cache = user_interface::Cache::default();
    let mut window_ids: HashMap<_, _> = windows
        .iter()
        .map(|(&id, window)| (window.id(), id))
        .collect();

    let mut states = HashMap::new();
    let mut surfaces = HashMap::new();
    let mut interfaces = ManuallyDrop::new(HashMap::new());

    for (&id, window) in windows.keys().zip(windows.values()) {
        let mut surface = compositor.create_surface(window);

        let state = State::new(&application, window);

        let physical_size = state.physical_size();

        compositor.configure_surface(
            &mut surface,
            physical_size.width,
            physical_size.height,
        );

        let user_interface = build_user_interface(
            &application,
            user_interface::Cache::default(),
            &mut renderer,
            state.logical_size(),
            &mut debug,
            id,
        );

        let _ = states.insert(id, state);
        let _ = surfaces.insert(id, surface);
        let _ = interfaces.insert(id, user_interface);
    }

    {
        // TODO(derezzedex)
        let state = states.values().next().expect("No state found");

        run_command(
            &application,
            &mut cache,
            state,
            &mut renderer,
            init_command,
            &mut runtime,
            &mut clipboard,
            &mut proxy,
            &mut debug,
            &windows,
            &window_ids,
            || compositor.fetch_information(),
        );
    }
    runtime.track(application.subscription().map(Event::Application));

    let mut mouse_interaction = mouse::Interaction::default();
    let mut events = Vec::new();
    let mut messages = Vec::new();

    debug.startup_finished();

    'main: while let Some(event) = receiver.next().await {
        match event {
            event::Event::MainEventsCleared => {
                for id in states.keys().copied().collect::<Vec<_>>() {
                    let (filtered, remaining): (Vec<_>, Vec<_>) =
                        events.iter().cloned().partition(
                            |(window_id, _event): &(
                                Option<crate::window::Id>,
                                iced_native::event::Event,
                            )| {
                                *window_id == Some(id) || *window_id == None
                            },
                        );

                    events.retain(|el| remaining.contains(el));
                    let filtered: Vec<_> = filtered
                        .into_iter()
                        .map(|(_id, event)| event)
                        .collect();

                    let cursor_position =
                        states.get(&id).unwrap().cursor_position();
                    let window = windows.get(&id).unwrap();

                    if filtered.is_empty() && messages.is_empty() {
                        continue;
                    }

                    debug.event_processing_started();

                    let (interface_state, statuses) = {
                        let user_interface = interfaces.get_mut(&id).unwrap();
                        user_interface.update(
                            &filtered,
                            cursor_position,
                            &mut renderer,
                            &mut clipboard,
                            &mut messages,
                        )
                    };

                    debug.event_processing_finished();

                    for event in filtered.into_iter().zip(statuses.into_iter())
                    {
                        runtime.broadcast(event);
                    }

                    // TODO(derezzedex): Should we redraw every window? We can't know what changed.
                    if !messages.is_empty()
                        || matches!(
                            interface_state,
                            user_interface::State::Outdated,
                        )
                    {
                        let state = &mut states.get_mut(&id).unwrap();
                        let pure_states: HashMap<_, _> =
                            ManuallyDrop::into_inner(interfaces)
                                .drain()
                                .map(
                                    |(id, interface): (
                                        window::Id,
                                        UserInterface<'_, _, _>,
                                    )| {
                                        (id, interface.into_cache())
                                    },
                                )
                                .collect();

                        // Update application
                        update(
                            &mut application,
                            &mut cache,
                            state,
                            &mut renderer,
                            &mut runtime,
                            &mut clipboard,
                            &mut proxy,
                            &mut debug,
                            &mut messages,
                            &windows,
                            &window_ids,
                            || compositor.fetch_information(),
                        );

                        // Update window
                        state.synchronize(&application, &windows, &proxy);

                        let should_exit = application.should_exit();

                        interfaces = ManuallyDrop::new(build_user_interfaces(
                            &application,
                            &mut renderer,
                            &mut debug,
                            &states,
                            pure_states,
                        ));

                        if should_exit {
                            break 'main;
                        }
                    }

                    debug.draw_started();
                    let new_mouse_interaction = {
                        let user_interface = interfaces.get_mut(&id).unwrap();
                        let state = states.get(&id).unwrap();

                        user_interface.draw(
                            &mut renderer,
                            state.theme(),
                            &renderer::Style {
                                text_color: state.text_color(),
                            },
                            state.cursor_position(),
                        )
                    };
                    debug.draw_finished();

                    if new_mouse_interaction != mouse_interaction {
                        window.set_cursor_icon(conversion::mouse_interaction(
                            new_mouse_interaction,
                        ));

                        mouse_interaction = new_mouse_interaction;
                    }

                    for window in windows.values() {
                        window.request_redraw();
                    }
                }
            }
            event::Event::PlatformSpecific(event::PlatformSpecific::MacOS(
                event::MacOS::ReceivedUrl(url),
            )) => {
                use iced_native::event;
                events.push((
                    None,
                    iced_native::Event::PlatformSpecific(
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
                Event::WindowCreated(id, window) => {
                    let mut surface = compositor.create_surface(&window);

                    let state = State::new(&application, &window);

                    let physical_size = state.physical_size();

                    compositor.configure_surface(
                        &mut surface,
                        physical_size.width,
                        physical_size.height,
                    );

                    let user_interface = build_user_interface(
                        &application,
                        user_interface::Cache::default(),
                        &mut renderer,
                        state.logical_size(),
                        &mut debug,
                        id,
                    );

                    let _ = states.insert(id, state);
                    let _ = surfaces.insert(id, surface);
                    let _ = interfaces.insert(id, user_interface);
                    let _ = window_ids.insert(window.id(), id);
                    let _ = windows.insert(id, window);
                }
                Event::CloseWindow(id) => {
                    // TODO(derezzedex): log errors
                    if let Some(window) = windows.get(&id) {
                        if window_ids.remove(&window.id()).is_none() {
                            println!("Failed to remove from `window_ids`!");
                        }
                    }
                    if states.remove(&id).is_none() {
                        println!("Failed to remove from `states`!")
                    }
                    if interfaces.remove(&id).is_none() {
                        println!("Failed to remove from `interfaces`!");
                    }
                    if windows.remove(&id).is_none() {
                        println!("Failed to remove from `windows`!")
                    }
                    if surfaces.remove(&id).is_none() {
                        println!("Failed to remove from `surfaces`!")
                    }

                    if windows.is_empty() {
                        break 'main;
                    }
                }
                Event::NewWindow(_, _) => unreachable!(),
            },
            event::Event::RedrawRequested(id) => {
                let state = window_ids
                    .get(&id)
                    .and_then(|id| states.get_mut(id))
                    .unwrap();
                let surface = window_ids
                    .get(&id)
                    .and_then(|id| surfaces.get_mut(id))
                    .unwrap();
                let physical_size = state.physical_size();

                if physical_size.width == 0 || physical_size.height == 0 {
                    continue;
                }

                debug.render_started();

                if state.viewport_changed() {
                    let mut user_interface = window_ids
                        .get(&id)
                        .and_then(|id| interfaces.remove(id))
                        .unwrap();

                    let logical_size = state.logical_size();

                    debug.layout_started();
                    user_interface =
                        user_interface.relayout(logical_size, &mut renderer);
                    debug.layout_finished();

                    debug.draw_started();
                    let new_mouse_interaction = {
                        let state = &state;

                        user_interface.draw(
                            &mut renderer,
                            state.theme(),
                            &renderer::Style {
                                text_color: state.text_color(),
                            },
                            state.cursor_position(),
                        )
                    };

                    let window = window_ids
                        .get(&id)
                        .and_then(|id| windows.get(id))
                        .unwrap();
                    if new_mouse_interaction != mouse_interaction {
                        window.set_cursor_icon(conversion::mouse_interaction(
                            new_mouse_interaction,
                        ));

                        mouse_interaction = new_mouse_interaction;
                    }
                    debug.draw_finished();

                    let _ = interfaces
                        .insert(*window_ids.get(&id).unwrap(), user_interface);

                    compositor.configure_surface(
                        surface,
                        physical_size.width,
                        physical_size.height,
                    );
                }

                match compositor.present(
                    &mut renderer,
                    surface,
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

                            // Try rendering again next frame.
                            // TODO(derezzedex)
                            windows
                                .values()
                                .next()
                                .expect("No window found")
                                .request_redraw();
                        }
                    },
                }
            }
            event::Event::WindowEvent {
                event: window_event,
                window_id,
            } => {
                // dbg!(window_id);
                if let Some(window) =
                    window_ids.get(&window_id).and_then(|id| windows.get(id))
                {
                    if let Some(state) = window_ids
                        .get(&window_id)
                        .and_then(|id| states.get_mut(id))
                    {
                        if requests_exit(&window_event, state.modifiers()) {
                            if let Some(id) =
                                window_ids.get(&window_id).cloned()
                            {
                                let message = application.close_requested(id);
                                messages.push(message);
                            }
                        }

                        state.update(window, &window_event, &mut debug);

                        if let Some(event) = conversion::window_event(
                            &window_event,
                            state.scale_factor(),
                            state.modifiers(),
                        ) {
                            events.push((
                                window_ids.get(&window_id).cloned(),
                                event,
                            ));
                        }
                    } else {
                        // TODO(derezzedex): log error
                    }
                } else {
                    // TODO(derezzedex): log error
                    // println!("{:?}: {:?}", window_id, window_event);
                }
            }
            _ => {}
        }
    }

    // Manually drop the user interface
    // drop(ManuallyDrop::into_inner(user_interface));
}

/// Returns true if the provided event should cause an [`Application`] to
/// exit.
pub fn requests_exit(
    event: &winit::event::WindowEvent<'_>,
    _modifiers: winit::event::ModifiersState,
) -> bool {
    use winit::event::WindowEvent;

    match event {
        WindowEvent::CloseRequested => true,
        #[cfg(target_os = "macos")]
        WindowEvent::KeyboardInput {
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

/// Builds a [`UserInterface`] for the provided [`Application`], logging
/// [`struct@Debug`] information accordingly.
pub fn build_user_interface<'a, A: Application>(
    application: &'a A,
    cache: user_interface::Cache,
    renderer: &mut A::Renderer,
    size: Size,
    debug: &mut Debug,
    id: window::Id,
) -> UserInterface<'a, A::Message, A::Renderer>
where
    <A::Renderer as crate::Renderer>::Theme: StyleSheet,
{
    debug.view_started();
    let view = application.view(id);
    debug.view_finished();

    debug.layout_started();
    let user_interface = UserInterface::build(view, size, cache, renderer);
    debug.layout_finished();

    user_interface
}

/// Updates an [`Application`] by feeding it the provided messages, spawning any
/// resulting [`Command`], and tracking its [`Subscription`].
pub fn update<A: Application, E: Executor>(
    application: &mut A,
    cache: &mut user_interface::Cache,
    state: &State<A>,
    renderer: &mut A::Renderer,
    runtime: &mut Runtime<E, Proxy<Event<A::Message>>, Event<A::Message>>,
    clipboard: &mut Clipboard,
    proxy: &mut winit::event_loop::EventLoopProxy<Event<A::Message>>,
    debug: &mut Debug,
    messages: &mut Vec<A::Message>,
    windows: &HashMap<window::Id, winit::window::Window>,
    window_ids: &HashMap<winit::window::WindowId, window::Id>,
    graphics_info: impl FnOnce() -> compositor::Information + Copy,
) where
    <A::Renderer as crate::Renderer>::Theme: StyleSheet,
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
            clipboard,
            proxy,
            debug,
            windows,
            window_ids,
            graphics_info,
        );
    }

    let subscription = application.subscription().map(Event::Application);
    runtime.track(subscription);
}

/// Runs the actions of a [`Command`].
pub fn run_command<A, E>(
    application: &A,
    cache: &mut user_interface::Cache,
    state: &State<A>,
    renderer: &mut A::Renderer,
    command: Command<A::Message>,
    runtime: &mut Runtime<E, Proxy<Event<A::Message>>, Event<A::Message>>,
    clipboard: &mut Clipboard,
    proxy: &mut winit::event_loop::EventLoopProxy<Event<A::Message>>,
    debug: &mut Debug,
    windows: &HashMap<window::Id, winit::window::Window>,
    window_ids: &HashMap<winit::window::WindowId, window::Id>,
    _graphics_info: impl FnOnce() -> compositor::Information + Copy,
) where
    A: Application,
    E: Executor,
    <A::Renderer as crate::Renderer>::Theme: StyleSheet,
{
    use iced_native::command;
    use iced_native::system;
    use iced_native::window;

    // TODO(derezzedex)
    let window = windows.values().next().expect("No window found");
    let id = *window_ids.get(&window.id()).unwrap();

    for action in command.actions() {
        match action {
            command::Action::Future(future) => {
                runtime.spawn(Box::pin(future.map(Event::Application)));
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
            command::Action::Window(action) => match action {
                window::Action::Resize { width, height } => {
                    window.set_inner_size(winit::dpi::LogicalSize {
                        width,
                        height,
                    });
                }
                window::Action::Move { x, y } => {
                    window.set_outer_position(winit::dpi::LogicalPosition {
                        x,
                        y,
                    });
                }
                window::Action::SetMode(mode) => {
                    window.set_visible(conversion::visible(mode));
                    window.set_fullscreen(conversion::fullscreen(
                        window.primary_monitor(),
                        mode,
                    ));
                }
                window::Action::FetchMode(tag) => {
                    let mode = if window.is_visible().unwrap_or(true) {
                        conversion::mode(window.fullscreen())
                    } else {
                        window::Mode::Hidden
                    };

                    proxy
                        .send_event(Event::Application(tag(mode)))
                        .expect("Send message to event loop");
                }
            },
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
                                .send_event(Event::Application(message))
                                .expect("Send message to event loop")
                        });
                    }
                }
            },
            command::Action::Widget(action) => {
                let mut current_cache = std::mem::take(cache);
                let mut current_operation = Some(action.into_operation());

                let mut user_interface = build_user_interface(
                    application,
                    current_cache,
                    renderer,
                    state.logical_size(),
                    debug,
                    id,
                );

                while let Some(mut operation) = current_operation.take() {
                    user_interface.operate(renderer, operation.as_mut());

                    match operation.finish() {
                        operation::Outcome::None => {}
                        operation::Outcome::Some(message) => {
                            proxy
                                .send_event(Event::Application(message))
                                .expect("Send message to event loop");
                        }
                        operation::Outcome::Chain(next) => {
                            current_operation = Some(next);
                        }
                    }
                }

                current_cache = user_interface.into_cache();
                *cache = current_cache;
            }
        }
    }
}

/// TODO(derezzedex)
pub fn build_user_interfaces<'a, A>(
    application: &'a A,
    renderer: &mut A::Renderer,
    debug: &mut Debug,
    states: &HashMap<window::Id, State<A>>,
    mut pure_states: HashMap<window::Id, user_interface::Cache>,
) -> HashMap<
    window::Id,
    UserInterface<
        'a,
        <A as Application>::Message,
        <A as Application>::Renderer,
    >,
>
where
    A: Application + 'static,
    <A::Renderer as crate::Renderer>::Theme: StyleSheet,
{
    let mut interfaces = HashMap::new();

    for (id, pure_state) in pure_states.drain() {
        let state = &states.get(&id).unwrap();

        let user_interface = build_user_interface(
            application,
            pure_state,
            renderer,
            state.logical_size(),
            debug,
            id,
        );

        let _ = interfaces.insert(id, user_interface);
    }

    interfaces
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
