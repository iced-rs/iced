//! Create interactive, native cross-platform applications.
mod state;

pub use state::State;

use crate::conversion;
use crate::core;
use crate::core::mouse;
use crate::core::renderer;
use crate::core::time::Instant;
use crate::core::widget::operation;
use crate::core::window;
use crate::core::{Event, Size};
use crate::futures::futures;
use crate::futures::{Executor, Runtime, Subscription};
use crate::graphics::compositor::{self, Compositor};
use crate::runtime::clipboard;
use crate::runtime::program::Program;
use crate::runtime::user_interface::{self, UserInterface};
use crate::runtime::{Command, Debug};
use crate::style::application::{Appearance, StyleSheet};
use crate::{Clipboard, Error, Proxy, Settings};

use futures::channel::mpsc;

use std::mem::ManuallyDrop;
use std::sync::Arc;

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
    fn title(&self) -> String;

    /// Returns the current `Theme` of the [`Application`].
    fn theme(&self) -> <Self::Renderer as core::Renderer>::Theme;

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
    use futures::task;
    use futures::Future;
    use winit::event_loop::EventLoopBuilder;

    let mut debug = Debug::new();
    debug.startup_started();

    let event_loop = EventLoopBuilder::with_user_event()
        .build()
        .expect("Create event loop");
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

    #[cfg(target_arch = "wasm32")]
    let target = settings.window.platform_specific.target.clone();

    let should_be_visible = settings.window.visible;
    let exit_on_close_request = settings.window.exit_on_close_request;

    let builder = conversion::window_settings(
        settings.window,
        &application.title(),
        event_loop.primary_monitor(),
        settings.id,
    )
    .with_visible(false);

    log::debug!("Window builder: {builder:#?}");

    let window = Arc::new(
        builder
            .build(&event_loop)
            .map_err(Error::WindowCreationFailed)?,
    );

    #[cfg(target_arch = "wasm32")]
    {
        use winit::platform::web::WindowExtWebSys;

        let canvas = window.canvas().expect("Get window canvas");

        let window = web_sys::window().unwrap();
        let document = window.document().unwrap();
        let body = document.body().unwrap();

        let target = target.and_then(|target| {
            body.query_selector(&format!("#{target}"))
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

    let compositor = C::new(compositor_settings, window.clone())?;
    let mut renderer = compositor.create_renderer();

    for font in settings.fonts {
        use crate::core::text::Renderer;

        renderer.load_font(font);
    }

    let (mut event_sender, event_receiver) = mpsc::unbounded();
    let (control_sender, mut control_receiver) = mpsc::unbounded();

    let mut instance = Box::pin(run_instance::<A, E, C>(
        application,
        compositor,
        renderer,
        runtime,
        proxy,
        debug,
        event_receiver,
        control_sender,
        init_command,
        window,
        should_be_visible,
        exit_on_close_request,
    ));

    let mut context = task::Context::from_waker(task::noop_waker_ref());

    let _ = event_loop.run(move |event, event_loop| {
        if event_loop.exiting() {
            return;
        }

        event_sender.start_send(event).expect("Send event");

        let poll = instance.as_mut().poll(&mut context);

        match poll {
            task::Poll::Pending => {
                if let Ok(Some(flow)) = control_receiver.try_next() {
                    event_loop.set_control_flow(flow);
                }
            }
            task::Poll::Ready(_) => {
                event_loop.exit();
            }
        };
    });

    Ok(())
}

async fn run_instance<A, E, C>(
    mut application: A,
    mut compositor: C,
    mut renderer: A::Renderer,
    mut runtime: Runtime<E, Proxy<A::Message>, A::Message>,
    mut proxy: winit::event_loop::EventLoopProxy<A::Message>,
    mut debug: Debug,
    mut event_receiver: mpsc::UnboundedReceiver<
        winit::event::Event<A::Message>,
    >,
    mut control_sender: mpsc::UnboundedSender<winit::event_loop::ControlFlow>,
    init_command: Command<A::Message>,
    window: Arc<winit::window::Window>,
    should_be_visible: bool,
    exit_on_close_request: bool,
) where
    A: Application + 'static,
    E: Executor + 'static,
    C: Compositor<Renderer = A::Renderer> + 'static,
    <A::Renderer as core::Renderer>::Theme: StyleSheet,
{
    use futures::stream::StreamExt;
    use winit::event;
    use winit::event_loop::ControlFlow;

    let mut state = State::new(&application, &window);
    let mut viewport_version = state.viewport_version();
    let physical_size = state.physical_size();

    let mut clipboard = Clipboard::connect(&window);
    let mut cache = user_interface::Cache::default();
    let mut surface = compositor.create_surface(
        window.clone(),
        physical_size.width,
        physical_size.height,
    );
    let mut should_exit = false;

    if should_be_visible {
        window.set_visible(true);
    }

    run_command(
        &application,
        &mut compositor,
        &mut surface,
        &mut cache,
        &state,
        &mut renderer,
        init_command,
        &mut runtime,
        &mut clipboard,
        &mut should_exit,
        &mut proxy,
        &mut debug,
        &window,
    );
    runtime.track(application.subscription().into_recipes());

    let mut user_interface = ManuallyDrop::new(build_user_interface(
        &application,
        cache,
        &mut renderer,
        state.logical_size(),
        &mut debug,
    ));

    let mut mouse_interaction = mouse::Interaction::default();
    let mut events = Vec::new();
    let mut messages = Vec::new();
    let mut redraw_pending = false;

    debug.startup_finished();

    while let Some(event) = event_receiver.next().await {
        match event {
            event::Event::NewEvents(
                event::StartCause::Init
                | event::StartCause::ResumeTimeReached { .. },
            ) if !redraw_pending => {
                window.request_redraw();
                redraw_pending = true;
            }
            event::Event::PlatformSpecific(event::PlatformSpecific::MacOS(
                event::MacOS::ReceivedUrl(url),
            )) => {
                use crate::core::event;

                events.push(Event::PlatformSpecific(
                    event::PlatformSpecific::MacOS(event::MacOS::ReceivedUrl(
                        url,
                    )),
                ));
            }
            event::Event::UserEvent(message) => {
                messages.push(message);
            }
            event::Event::WindowEvent {
                event: event::WindowEvent::RedrawRequested { .. },
                ..
            } => {
                let physical_size = state.physical_size();

                if physical_size.width == 0 || physical_size.height == 0 {
                    continue;
                }

                let current_viewport_version = state.viewport_version();

                if viewport_version != current_viewport_version {
                    let logical_size = state.logical_size();

                    debug.layout_started();
                    user_interface = ManuallyDrop::new(
                        ManuallyDrop::into_inner(user_interface)
                            .relayout(logical_size, &mut renderer),
                    );
                    debug.layout_finished();

                    compositor.configure_surface(
                        &mut surface,
                        physical_size.width,
                        physical_size.height,
                    );

                    viewport_version = current_viewport_version;
                }

                // TODO: Avoid redrawing all the time by forcing widgets to
                // request redraws on state changes
                //
                // Then, we can use the `interface_state` here to decide if a redraw
                // is needed right away, or simply wait until a specific time.
                let redraw_event = Event::Window(
                    window::Id::MAIN,
                    window::Event::RedrawRequested(Instant::now()),
                );

                let (interface_state, _) = user_interface.update(
                    &[redraw_event.clone()],
                    state.cursor(),
                    &mut renderer,
                    &mut clipboard,
                    &mut messages,
                );

                let _ = control_sender.start_send(match interface_state {
                    user_interface::State::Updated {
                        redraw_request: Some(redraw_request),
                    } => match redraw_request {
                        window::RedrawRequest::NextFrame => {
                            window.request_redraw();

                            ControlFlow::Wait
                        }
                        window::RedrawRequest::At(at) => {
                            ControlFlow::WaitUntil(at)
                        }
                    },
                    _ => ControlFlow::Wait,
                });

                runtime.broadcast(redraw_event, core::event::Status::Ignored);

                debug.draw_started();
                let new_mouse_interaction = user_interface.draw(
                    &mut renderer,
                    state.theme(),
                    &renderer::Style {
                        text_color: state.text_color(),
                    },
                    state.cursor(),
                );
                redraw_pending = false;
                debug.draw_finished();

                if new_mouse_interaction != mouse_interaction {
                    window.set_cursor_icon(conversion::mouse_interaction(
                        new_mouse_interaction,
                    ));

                    mouse_interaction = new_mouse_interaction;
                }

                debug.render_started();
                match compositor.present(
                    &mut renderer,
                    &mut surface,
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
                            panic!("{error:?}");
                        }
                        _ => {
                            debug.render_finished();

                            // Try rendering again next frame.
                            window.request_redraw();
                        }
                    },
                }
            }
            event::Event::WindowEvent {
                event: window_event,
                ..
            } => {
                if requests_exit(&window_event, state.modifiers())
                    && exit_on_close_request
                {
                    break;
                }

                state.update(&window, &window_event, &mut debug);

                if let Some(event) = conversion::window_event(
                    window::Id::MAIN,
                    window_event,
                    state.scale_factor(),
                    state.modifiers(),
                ) {
                    events.push(event);
                }
            }
            event::Event::AboutToWait => {
                if events.is_empty() && messages.is_empty() {
                    continue;
                }

                debug.event_processing_started();

                let (interface_state, statuses) = user_interface.update(
                    &events,
                    state.cursor(),
                    &mut renderer,
                    &mut clipboard,
                    &mut messages,
                );

                debug.event_processing_finished();

                for (event, status) in
                    events.drain(..).zip(statuses.into_iter())
                {
                    runtime.broadcast(event, status);
                }

                if !messages.is_empty()
                    || matches!(
                        interface_state,
                        user_interface::State::Outdated
                    )
                {
                    let mut cache =
                        ManuallyDrop::into_inner(user_interface).into_cache();

                    // Update application
                    update(
                        &mut application,
                        &mut compositor,
                        &mut surface,
                        &mut cache,
                        &mut state,
                        &mut renderer,
                        &mut runtime,
                        &mut clipboard,
                        &mut should_exit,
                        &mut proxy,
                        &mut debug,
                        &mut messages,
                        &window,
                    );

                    user_interface = ManuallyDrop::new(build_user_interface(
                        &application,
                        cache,
                        &mut renderer,
                        state.logical_size(),
                        &mut debug,
                    ));

                    if should_exit {
                        break;
                    }
                }

                if !redraw_pending {
                    window.request_redraw();
                    redraw_pending = true;
                }
            }
            _ => {}
        }
    }

    // Manually drop the user interface
    drop(ManuallyDrop::into_inner(user_interface));
}

/// Returns true if the provided event should cause an [`Application`] to
/// exit.
pub fn requests_exit(
    event: &winit::event::WindowEvent,
    _modifiers: winit::keyboard::ModifiersState,
) -> bool {
    use winit::event::WindowEvent;

    match event {
        WindowEvent::CloseRequested => true,
        #[cfg(target_os = "macos")]
        WindowEvent::KeyboardInput {
            event:
                winit::event::KeyEvent {
                    logical_key: winit::keyboard::Key::Character(c),
                    state: winit::event::ElementState::Pressed,
                    ..
                },
            ..
        } if c == "q" && _modifiers.super_key() => true,
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
) -> UserInterface<'a, A::Message, A::Renderer>
where
    <A::Renderer as core::Renderer>::Theme: StyleSheet,
{
    debug.view_started();
    let view = application.view();
    debug.view_finished();

    debug.layout_started();
    let user_interface = UserInterface::build(view, size, cache, renderer);
    debug.layout_finished();

    user_interface
}

/// Updates an [`Application`] by feeding it the provided messages, spawning any
/// resulting [`Command`], and tracking its [`Subscription`].
pub fn update<A: Application, C, E: Executor>(
    application: &mut A,
    compositor: &mut C,
    surface: &mut C::Surface,
    cache: &mut user_interface::Cache,
    state: &mut State<A>,
    renderer: &mut A::Renderer,
    runtime: &mut Runtime<E, Proxy<A::Message>, A::Message>,
    clipboard: &mut Clipboard,
    should_exit: &mut bool,
    proxy: &mut winit::event_loop::EventLoopProxy<A::Message>,
    debug: &mut Debug,
    messages: &mut Vec<A::Message>,
    window: &winit::window::Window,
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
            surface,
            cache,
            state,
            renderer,
            command,
            runtime,
            clipboard,
            should_exit,
            proxy,
            debug,
            window,
        );
    }

    state.synchronize(application, window);

    let subscription = application.subscription();
    runtime.track(subscription.into_recipes());
}

/// Runs the actions of a [`Command`].
pub fn run_command<A, C, E>(
    application: &A,
    compositor: &mut C,
    surface: &mut C::Surface,
    cache: &mut user_interface::Cache,
    state: &State<A>,
    renderer: &mut A::Renderer,
    command: Command<A::Message>,
    runtime: &mut Runtime<E, Proxy<A::Message>, A::Message>,
    clipboard: &mut Clipboard,
    should_exit: &mut bool,
    proxy: &mut winit::event_loop::EventLoopProxy<A::Message>,
    debug: &mut Debug,
    window: &winit::window::Window,
) where
    A: Application,
    E: Executor,
    C: Compositor<Renderer = A::Renderer> + 'static,
    <A::Renderer as core::Renderer>::Theme: StyleSheet,
{
    use crate::runtime::command;
    use crate::runtime::system;
    use crate::runtime::window;

    for action in command.actions() {
        match action {
            command::Action::Future(future) => {
                runtime.spawn(future);
            }
            command::Action::Stream(stream) => {
                runtime.run(stream);
            }
            command::Action::Clipboard(action) => match action {
                clipboard::Action::Read(tag) => {
                    let message = tag(clipboard.read());

                    proxy
                        .send_event(message)
                        .expect("Send message to event loop");
                }
                clipboard::Action::Write(contents) => {
                    clipboard.write(contents);
                }
            },
            command::Action::Window(action) => match action {
                window::Action::Close(_id) => {
                    *should_exit = true;
                }
                window::Action::Drag(_id) => {
                    let _res = window.drag_window();
                }
                window::Action::Spawn { .. } => {
                    log::warn!(
                        "Spawning a window is only available with \
                        multi-window applications."
                    );
                }
                window::Action::Resize(_id, size) => {
                    let _ =
                        window.request_inner_size(winit::dpi::LogicalSize {
                            width: size.width,
                            height: size.height,
                        });
                }
                window::Action::FetchSize(_id, callback) => {
                    let size =
                        window.inner_size().to_logical(window.scale_factor());

                    proxy
                        .send_event(callback(Size::new(
                            size.width,
                            size.height,
                        )))
                        .expect("Send message to event loop");
                }
                window::Action::FetchMaximized(_id, callback) => {
                    proxy
                        .send_event(callback(window.is_maximized()))
                        .expect("Send message to event loop");
                }
                window::Action::Maximize(_id, maximized) => {
                    window.set_maximized(maximized);
                }
                window::Action::FetchMinimized(_id, callback) => {
                    proxy
                        .send_event(callback(window.is_minimized()))
                        .expect("Send message to event loop");
                }
                window::Action::Minimize(_id, minimized) => {
                    window.set_minimized(minimized);
                }
                window::Action::Move(_id, position) => {
                    window.set_outer_position(winit::dpi::LogicalPosition {
                        x: position.x,
                        y: position.y,
                    });
                }
                window::Action::ChangeMode(_id, mode) => {
                    window.set_visible(conversion::visible(mode));
                    window.set_fullscreen(conversion::fullscreen(
                        window.current_monitor(),
                        mode,
                    ));
                }
                window::Action::ChangeIcon(_id, icon) => {
                    window.set_window_icon(conversion::icon(icon));
                }
                window::Action::FetchMode(_id, tag) => {
                    let mode = if window.is_visible().unwrap_or(true) {
                        conversion::mode(window.fullscreen())
                    } else {
                        core::window::Mode::Hidden
                    };

                    proxy
                        .send_event(tag(mode))
                        .expect("Send message to event loop");
                }
                window::Action::ToggleMaximize(_id) => {
                    window.set_maximized(!window.is_maximized());
                }
                window::Action::ToggleDecorations(_id) => {
                    window.set_decorations(!window.is_decorated());
                }
                window::Action::RequestUserAttention(_id, user_attention) => {
                    window.request_user_attention(
                        user_attention.map(conversion::user_attention),
                    );
                }
                window::Action::GainFocus(_id) => {
                    window.focus_window();
                }
                window::Action::ChangeLevel(_id, level) => {
                    window.set_window_level(conversion::window_level(level));
                }
                window::Action::FetchId(_id, tag) => {
                    proxy
                        .send_event(tag(window.id().into()))
                        .expect("Send message to event loop");
                }
                window::Action::Screenshot(_id, tag) => {
                    let bytes = compositor.screenshot(
                        renderer,
                        surface,
                        state.viewport(),
                        state.background_color(),
                        &debug.overlay(),
                    );

                    proxy
                        .send_event(tag(window::Screenshot::new(
                            bytes,
                            state.physical_size(),
                        )))
                        .expect("Send message to event loop.");
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
                                .send_event(message)
                                .expect("Send message to event loop");
                        });
                    }
                }
            },
            command::Action::Widget(action) => {
                let mut current_cache = std::mem::take(cache);
                let mut current_operation = Some(action);

                let mut user_interface = build_user_interface(
                    application,
                    current_cache,
                    renderer,
                    state.logical_size(),
                    debug,
                );

                while let Some(mut operation) = current_operation.take() {
                    user_interface.operate(renderer, operation.as_mut());

                    match operation.finish() {
                        operation::Outcome::None => {}
                        operation::Outcome::Some(message) => {
                            proxy
                                .send_event(message)
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
            command::Action::LoadFont { bytes, tagger } => {
                use crate::core::text::Renderer;

                // TODO: Error handling (?)
                renderer.load_font(bytes);

                proxy
                    .send_event(tagger(Ok(())))
                    .expect("Send message to event loop");
            }
        }
    }
}
