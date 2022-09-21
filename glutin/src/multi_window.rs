//! Create interactive, native cross-platform applications.
mod state;

pub use state::State;

use crate::mouse;
use crate::{Error, Executor, Runtime};

pub use iced_winit::multi_window::{self, Application, StyleSheet};

use iced_winit::conversion;
use iced_winit::futures;
use iced_winit::futures::channel::mpsc;
use iced_winit::renderer;
use iced_winit::settings;
use iced_winit::user_interface;
use iced_winit::window;
use iced_winit::{Clipboard, Command, Debug, Proxy, Settings};

use glutin::window::Window;
use std::collections::HashMap;
use std::mem::ManuallyDrop;

/// Runs an [`Application`] with an executor, compositor, and the provided
/// settings.
pub fn run<A, E, C>(
    settings: Settings<A::Flags>,
    compositor_settings: C::Settings,
) -> Result<(), Error>
where
    A: Application + 'static,
    E: Executor + 'static,
    C: iced_graphics::window::GLCompositor<Renderer = A::Renderer> + 'static,
    <A::Renderer as iced_native::Renderer>::Theme: StyleSheet,
{
    use futures::task;
    use futures::Future;
    use glutin::event_loop::EventLoopBuilder;
    use glutin::platform::run_return::EventLoopExtRunReturn;
    use glutin::ContextBuilder;

    let mut debug = Debug::new();
    debug.startup_started();

    let mut event_loop = EventLoopBuilder::with_user_event().build();
    let proxy = event_loop.create_proxy();

    let runtime = {
        let executor = E::new().map_err(Error::ExecutorCreationFailed)?;
        let proxy = Proxy::new(event_loop.create_proxy());

        Runtime::new(executor, proxy)
    };

    let (application, init_command) = {
        let flags = settings.flags;

        runtime.enter(|| A::new(flags))
    };

    let context = {
        let builder = settings.window.into_builder(
            &application.title(),
            event_loop.primary_monitor(),
            settings.id,
        );

        log::info!("Window builder: {:#?}", builder);

        let opengl_builder = ContextBuilder::new()
            .with_vsync(true)
            .with_multisampling(C::sample_count(&compositor_settings) as u16);

        let opengles_builder = opengl_builder.clone().with_gl(
            glutin::GlRequest::Specific(glutin::Api::OpenGlEs, (2, 0)),
        );

        let (first_builder, second_builder) = if settings.try_opengles_first {
            (opengles_builder, opengl_builder)
        } else {
            (opengl_builder, opengles_builder)
        };

        log::info!("Trying first builder: {:#?}", first_builder);

        let context = first_builder
            .build_windowed(builder.clone(), &event_loop)
            .or_else(|_| {
                log::info!("Trying second builder: {:#?}", second_builder);
                second_builder.build_windowed(builder, &event_loop)
            })
            .map_err(|error| {
                use glutin::CreationError;
                use iced_graphics::Error as ContextError;

                match error {
                    CreationError::Window(error) => {
                        Error::WindowCreationFailed(error)
                    }
                    CreationError::OpenGlVersionNotSupported => {
                        Error::GraphicsCreationFailed(
                            ContextError::VersionNotSupported,
                        )
                    }
                    CreationError::NoAvailablePixelFormat => {
                        Error::GraphicsCreationFailed(
                            ContextError::NoAvailablePixelFormat,
                        )
                    }
                    error => Error::GraphicsCreationFailed(
                        ContextError::BackendError(error.to_string()),
                    ),
                }
            })?;

        #[allow(unsafe_code)]
        unsafe {
            context.make_current().expect("Make OpenGL context current")
        }
    };

    #[allow(unsafe_code)]
    let (compositor, renderer) = unsafe {
        C::new(compositor_settings, |address| {
            context.get_proc_address(address)
        })?
    };

    let (mut sender, receiver) = mpsc::unbounded();

    let mut instance = Box::pin(run_instance::<A, E, C>(
        application,
        compositor,
        renderer,
        runtime,
        proxy,
        debug,
        receiver,
        context,
        init_command,
        settings.exit_on_close_request,
    ));

    let mut context = task::Context::from_waker(task::noop_waker_ref());

    let _ = event_loop.run_return(move |event, event_loop, control_flow| {
        use glutin::event_loop::ControlFlow;

        if let ControlFlow::ExitWithCode(_) = control_flow {
            return;
        }

        let event = match event {
            glutin::event::Event::WindowEvent {
                event:
                    glutin::event::WindowEvent::ScaleFactorChanged {
                        new_inner_size,
                        ..
                    },
                window_id,
            } => Some(glutin::event::Event::WindowEvent {
                event: glutin::event::WindowEvent::Resized(*new_inner_size),
                window_id,
            }),
            glutin::event::Event::UserEvent(Event::NewWindow(id, settings)) => {
                // TODO(derezzedex)
                let window = settings
                    .into_builder(
                        "fix window title",
                        event_loop.primary_monitor(),
                        None,
                    )
                    .build(event_loop)
                    .expect("Failed to build window");

                Some(glutin::event::Event::UserEvent(Event::WindowCreated(
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
    });

    Ok(())
}

async fn run_instance<A, E, C>(
    mut application: A,
    mut compositor: C,
    mut renderer: A::Renderer,
    mut runtime: Runtime<E, Proxy<Event<A::Message>>, Event<A::Message>>,
    mut proxy: glutin::event_loop::EventLoopProxy<Event<A::Message>>,
    mut debug: Debug,
    mut receiver: mpsc::UnboundedReceiver<
        glutin::event::Event<'_, Event<A::Message>>,
    >,
    context: glutin::ContextWrapper<glutin::PossiblyCurrent, Window>,
    init_command: Command<A::Message>,
    _exit_on_close_request: bool,
) where
    A: Application + 'static,
    E: Executor + 'static,
    C: iced_graphics::window::GLCompositor<Renderer = A::Renderer> + 'static,
    <A::Renderer as iced_native::Renderer>::Theme: StyleSheet,
{
    use glutin::event;
    use iced_winit::futures::stream::StreamExt;

    let mut clipboard = Clipboard::connect(context.window());
    let mut cache = user_interface::Cache::default();
    let state = State::new(&application, context.window());
    let user_interface = multi_window::build_user_interface(
        &application,
        user_interface::Cache::default(),
        &mut renderer,
        state.logical_size(),
        &mut debug,
        window::Id::MAIN,
    );

    #[allow(unsafe_code)]
    let (mut context, window) = unsafe { context.split() };

    let mut window_ids = HashMap::from([(window.id(), window::Id::MAIN)]);
    let mut windows = HashMap::from([(window::Id::MAIN, window)]);
    let mut states = HashMap::from([(window::Id::MAIN, state)]);
    let mut interfaces =
        ManuallyDrop::new(HashMap::from([(window::Id::MAIN, user_interface)]));

    {
        let state = states.get(&window::Id::MAIN).unwrap();

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
                for id in windows.keys().copied() {
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

                    if !messages.is_empty()
                        || matches!(
                            interface_state,
                            user_interface::State::Outdated
                        )
                    {
                        let state = &mut states.get_mut(&id).unwrap();
                        let pure_states: HashMap<_, _> =
                            ManuallyDrop::into_inner(interfaces)
                                .drain()
                                .map(|(id, interface)| {
                                    (id, interface.into_cache())
                                })
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

                    window.request_redraw();
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
                Event::Application(message) => messages.push(message),
                Event::WindowCreated(id, window) => {
                    let state = State::new(&application, &window);
                    let user_interface = multi_window::build_user_interface(
                        &application,
                        user_interface::Cache::default(),
                        &mut renderer,
                        state.logical_size(),
                        &mut debug,
                        id,
                    );

                    let _ = states.insert(id, state);
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

                debug.render_started();

                #[allow(unsafe_code)]
                unsafe {
                    if !context.is_current() {
                        context = context
                            .make_current()
                            .expect("Make OpenGL context current");
                    }
                }

                if state.viewport_changed() {
                    let physical_size = state.physical_size();
                    let logical_size = state.logical_size();

                    let mut user_interface = window_ids
                        .get(&id)
                        .and_then(|id| interfaces.remove(id))
                        .unwrap();

                    debug.layout_started();
                    user_interface =
                        user_interface.relayout(logical_size, &mut renderer);
                    debug.layout_finished();

                    debug.draw_started();
                    let new_mouse_interaction = user_interface.draw(
                        &mut renderer,
                        state.theme(),
                        &renderer::Style {
                            text_color: state.text_color(),
                        },
                        state.cursor_position(),
                    );
                    debug.draw_finished();

                    if new_mouse_interaction != mouse_interaction {
                        let window = window_ids
                            .get(&id)
                            .and_then(|id| windows.get_mut(id))
                            .unwrap();

                        window.set_cursor_icon(conversion::mouse_interaction(
                            new_mouse_interaction,
                        ));

                        mouse_interaction = new_mouse_interaction;
                    }

                    context.resize(glutin::dpi::PhysicalSize::new(
                        physical_size.width,
                        physical_size.height,
                    ));

                    compositor.resize_viewport(physical_size);

                    let _ = interfaces
                        .insert(*window_ids.get(&id).unwrap(), user_interface);
                }

                compositor.present(
                    &mut renderer,
                    state.viewport(),
                    state.background_color(),
                    &debug.overlay(),
                );

                context.swap_buffers().expect("Swap buffers");

                debug.render_finished();

                // TODO: Handle animations!
                // Maybe we can use `ControlFlow::WaitUntil` for this.
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
                        if multi_window::requests_exit(
                            &window_event,
                            state.modifiers(),
                        ) {
                            if let Some(id) =
                                window_ids.get(&window_id).cloned()
                            {
                                let message = application.close_requested(id);
                                messages.push(message);
                            }
                        }

                        state.update(window, &window_event, &mut debug);

                        if let Some(event) = conversion::window_event(
                            *window_ids.get(&window_id).unwrap(),
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

/// TODO(derezzedex):
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
    WindowCreated(window::Id, glutin::window::Window),
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
    proxy: &mut glutin::event_loop::EventLoopProxy<Event<A::Message>>,
    debug: &mut Debug,
    messages: &mut Vec<A::Message>,
    windows: &HashMap<window::Id, glutin::window::Window>,
    graphics_info: impl FnOnce() -> iced_graphics::compositor::Information + Copy,
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
    proxy: &mut glutin::event_loop::EventLoopProxy<Event<A::Message>>,
    debug: &mut Debug,
    windows: &HashMap<window::Id, glutin::window::Window>,
    _graphics_info: impl FnOnce() -> iced_graphics::compositor::Information + Copy,
) where
    A: Application,
    E: Executor,
    <A::Renderer as crate::Renderer>::Theme: StyleSheet,
{
    use iced_native::command;
    use iced_native::system;
    use iced_native::window;
    use iced_winit::clipboard;
    use iced_winit::futures::FutureExt;

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
            command::Action::Window(id, action) => {
                let window = windows.get(&id).expect("No window found");

                match action {
                    window::Action::Resize { width, height } => {
                        window.set_inner_size(glutin::dpi::LogicalSize {
                            width,
                            height,
                        });
                    }
                    window::Action::Move { x, y } => {
                        window.set_outer_position(
                            glutin::dpi::LogicalPosition { x, y },
                        );
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
                }
            }
            command::Action::System(action) => match action {
                system::Action::QueryInformation(_tag) => {
                    #[cfg(feature = "iced_winit/system")]
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
                use crate::widget::operation;

                let mut current_cache = std::mem::take(cache);
                let mut current_operation = Some(action.into_operation());

                let mut user_interface = multi_window::build_user_interface(
                    application,
                    current_cache,
                    renderer,
                    state.logical_size(),
                    debug,
                    window::Id::MAIN, // TODO(derezzedex): run the operation on every widget tree
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
    iced_winit::UserInterface<
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

        let user_interface = multi_window::build_user_interface(
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
