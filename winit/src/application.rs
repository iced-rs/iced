//! Create interactive, native cross-platform applications.
mod state;

pub use state::State;

use crate::clipboard::{self, Clipboard};
use crate::conversion;
use crate::mouse;
use crate::renderer;
use crate::widget::operation;
use crate::{
    Command, Debug, Error, Executor, Proxy, Runtime, Settings, Size,
    Subscription,
};

use iced_futures::futures;
use iced_futures::futures::channel::mpsc;
use iced_graphics::compositor;
use iced_graphics::window;
use iced_native::program::Program;
use iced_native::user_interface::{self, UserInterface};

pub use iced_native::application::{Appearance, StyleSheet};

use std::mem::ManuallyDrop;

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
    <Self::Renderer as crate::Renderer>::Theme: StyleSheet,
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
    C: window::Compositor<Renderer = A::Renderer> + 'static,
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
        window,
        settings.exit_on_close_request,
    ));

    let mut context = task::Context::from_waker(task::noop_waker_ref());

    platform::run(event_loop, move |event, _, control_flow| {
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
    mut runtime: Runtime<E, Proxy<A::Message>, A::Message>,
    mut proxy: winit::event_loop::EventLoopProxy<A::Message>,
    mut debug: Debug,
    mut receiver: mpsc::UnboundedReceiver<winit::event::Event<'_, A::Message>>,
    init_command: Command<A::Message>,
    window: winit::window::Window,
    exit_on_close_request: bool,
) where
    A: Application + 'static,
    E: Executor + 'static,
    C: window::Compositor<Renderer = A::Renderer> + 'static,
    <A::Renderer as crate::Renderer>::Theme: StyleSheet,
{
    use iced_futures::futures::stream::StreamExt;
    use winit::event;

    let mut clipboard = Clipboard::connect(&window);
    let mut cache = user_interface::Cache::default();
    let mut surface = compositor.create_surface(&window);

    let mut state = State::new(&application, &window);
    let mut viewport_version = state.viewport_version();

    let physical_size = state.physical_size();

    compositor.configure_surface(
        &mut surface,
        physical_size.width,
        physical_size.height,
    );

    run_command(
        &application,
        &mut cache,
        &state,
        &mut renderer,
        init_command,
        &mut runtime,
        &mut clipboard,
        &mut proxy,
        &mut debug,
        &window,
        || compositor.fetch_information(),
    );
    runtime.track(application.subscription());

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

    debug.startup_finished();

    while let Some(event) = receiver.next().await {
        match event {
            event::Event::MainEventsCleared => {
                if events.is_empty() && messages.is_empty() {
                    continue;
                }

                debug.event_processing_started();

                let (interface_state, statuses) = user_interface.update(
                    &events,
                    state.cursor_position(),
                    &mut renderer,
                    &mut clipboard,
                    &mut messages,
                );

                debug.event_processing_finished();

                for event in events.drain(..).zip(statuses.into_iter()) {
                    runtime.broadcast(event);
                }

                if !messages.is_empty()
                    || matches!(
                        interface_state,
                        user_interface::State::Outdated,
                    )
                {
                    let mut cache =
                        ManuallyDrop::into_inner(user_interface).into_cache();

                    // Update application
                    update(
                        &mut application,
                        &mut cache,
                        &state,
                        &mut renderer,
                        &mut runtime,
                        &mut clipboard,
                        &mut proxy,
                        &mut debug,
                        &mut messages,
                        &window,
                        || compositor.fetch_information(),
                    );

                    // Update window
                    state.synchronize(&application, &window);

                    let should_exit = application.should_exit();

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
                    window.set_cursor_icon(conversion::mouse_interaction(
                        new_mouse_interaction,
                    ));

                    mouse_interaction = new_mouse_interaction;
                }

                window.request_redraw();
            }
            event::Event::PlatformSpecific(event::PlatformSpecific::MacOS(
                event::MacOS::ReceivedUrl(url),
            )) => {
                use iced_native::event;

                events.push(iced_native::Event::PlatformSpecific(
                    event::PlatformSpecific::MacOS(event::MacOS::ReceivedUrl(
                        url,
                    )),
                ));
            }
            event::Event::UserEvent(message) => {
                messages.push(message);
            }
            event::Event::RedrawRequested(_) => {
                let physical_size = state.physical_size();

                if physical_size.width == 0 || physical_size.height == 0 {
                    continue;
                }

                debug.render_started();
                let current_viewport_version = state.viewport_version();

                if viewport_version != current_viewport_version {
                    let logical_size = state.logical_size();

                    debug.layout_started();
                    user_interface = ManuallyDrop::new(
                        ManuallyDrop::into_inner(user_interface)
                            .relayout(logical_size, &mut renderer),
                    );
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

                    if new_mouse_interaction != mouse_interaction {
                        window.set_cursor_icon(conversion::mouse_interaction(
                            new_mouse_interaction,
                        ));

                        mouse_interaction = new_mouse_interaction;
                    }
                    debug.draw_finished();

                    compositor.configure_surface(
                        &mut surface,
                        physical_size.width,
                        physical_size.height,
                    );

                    viewport_version = current_viewport_version;
                }

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
                            panic!("{:?}", error);
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
                    &window_event,
                    state.scale_factor(),
                    state.modifiers(),
                ) {
                    events.push(event);
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
) -> UserInterface<'a, A::Message, A::Renderer>
where
    <A::Renderer as crate::Renderer>::Theme: StyleSheet,
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
pub fn update<A: Application, E: Executor>(
    application: &mut A,
    cache: &mut user_interface::Cache,
    state: &State<A>,
    renderer: &mut A::Renderer,
    runtime: &mut Runtime<E, Proxy<A::Message>, A::Message>,
    clipboard: &mut Clipboard,
    proxy: &mut winit::event_loop::EventLoopProxy<A::Message>,
    debug: &mut Debug,
    messages: &mut Vec<A::Message>,
    window: &winit::window::Window,
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
            window,
            graphics_info,
        );
    }

    let subscription = application.subscription();
    runtime.track(subscription);
}

/// Runs the actions of a [`Command`].
pub fn run_command<A, E>(
    application: &A,
    cache: &mut user_interface::Cache,
    state: &State<A>,
    renderer: &mut A::Renderer,
    command: Command<A::Message>,
    runtime: &mut Runtime<E, Proxy<A::Message>, A::Message>,
    clipboard: &mut Clipboard,
    proxy: &mut winit::event_loop::EventLoopProxy<A::Message>,
    debug: &mut Debug,
    window: &winit::window::Window,
    _graphics_info: impl FnOnce() -> compositor::Information + Copy,
) where
    A: Application,
    E: Executor,
    <A::Renderer as crate::Renderer>::Theme: StyleSheet,
{
    use iced_native::command;
    use iced_native::system;
    use iced_native::window;

    for action in command.actions() {
        match action {
            command::Action::Future(future) => {
                runtime.spawn(future);
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
                window::Action::Drag => {
                    let _res = window.drag_window();
                }
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
                        .send_event(tag(mode))
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
                                .send_event(message)
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
        }
    }
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
