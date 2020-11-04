//! Create interactive, native cross-platform applications.
use crate::conversion;
use crate::mouse;
use crate::{
    Clipboard, Color, Command, Debug, Error, Executor, Mode, Proxy, Runtime,
    Settings, Size, Subscription,
};
use iced_graphics::window;
use iced_graphics::Viewport;
use iced_native::program::Program;
use iced_native::{Cache, UserInterface};

use iced_futures::futures;
use iced_futures::futures::channel::mpsc;

/// An interactive, native cross-platform application.
///
/// This trait is the main entrypoint of Iced. Once implemented, you can run
/// your GUI application by simply calling [`run`](#method.run). It will run in
/// its own window.
///
/// An [`Application`](trait.Application.html) can execute asynchronous actions
/// by returning a [`Command`](struct.Command.html) in some of its methods.
///
/// When using an [`Application`] with the `debug` feature enabled, a debug view
/// can be toggled by pressing `F12`.
///
/// [`Application`]: trait.Application.html
pub trait Application: Program {
    /// The data needed to initialize your [`Application`].
    ///
    /// [`Application`]: trait.Application.html
    type Flags;

    /// Initializes the [`Application`] with the flags provided to
    /// [`run`] as part of the [`Settings`].
    ///
    /// Here is where you should return the initial state of your app.
    ///
    /// Additionally, you can return a [`Command`](struct.Command.html) if you
    /// need to perform some async action in the background on startup. This is
    /// useful if you want to load state from a file, perform an initial HTTP
    /// request, etc.
    ///
    /// [`Application`]: trait.Application.html
    /// [`run`]: #method.run.html
    /// [`Settings`]: ../settings/struct.Settings.html
    fn new(flags: Self::Flags) -> (Self, Command<Self::Message>);

    /// Returns the current title of the [`Application`].
    ///
    /// This title can be dynamic! The runtime will automatically update the
    /// title of your application when necessary.
    ///
    /// [`Application`]: trait.Application.html
    fn title(&self) -> String;

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

    /// Returns the current [`Application`] mode.
    ///
    /// The runtime will automatically transition your application if a new mode
    /// is returned.
    ///
    /// By default, an application will run in windowed mode.
    ///
    /// [`Application`]: trait.Application.html
    fn mode(&self) -> Mode {
        Mode::Windowed
    }

    /// Returns the background [`Color`] of the [`Application`].
    ///
    /// By default, it returns [`Color::WHITE`].
    ///
    /// [`Color`]: struct.Color.html
    /// [`Application`]: trait.Application.html
    /// [`Color::WHITE`]: struct.Color.html#const.WHITE
    fn background_color(&self) -> Color {
        Color::WHITE
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
    ///
    /// [`Application`]: trait.Application.html
    fn scale_factor(&self) -> f64 {
        1.0
    }
}

/// Runs an [`Application`] with an executor, compositor, and the provided
/// settings.
///
/// [`Application`]: trait.Application.html
pub fn run<A, E, C>(
    settings: Settings<A::Flags>,
    compositor_settings: C::Settings,
) -> Result<(), Error>
where
    A: Application + 'static,
    E: Executor + 'static,
    C: window::Compositor<Renderer = A::Renderer> + 'static,
{
    use futures::task::Poll;
    use futures::Future;
    use winit::event_loop::EventLoop;

    let mut debug = Debug::new();
    debug.startup_started();

    let (compositor, renderer) = C::new(compositor_settings)?;

    let event_loop = EventLoop::with_user_event();

    let mut runtime = {
        let proxy = Proxy::new(event_loop.create_proxy());
        let executor = E::new().map_err(Error::ExecutorCreationFailed)?;

        Runtime::new(executor, proxy)
    };

    let (application, init_command) = {
        let flags = settings.flags;
        runtime.enter(|| A::new(flags))
    };

    let subscription = application.subscription();

    runtime.spawn(init_command);
    runtime.track(subscription);

    let window = settings
        .window
        .into_builder(
            &application.title(),
            application.mode(),
            event_loop.primary_monitor(),
        )
        .build(&event_loop)
        .map_err(Error::WindowCreationFailed)?;

    let (mut sender, receiver) = mpsc::unbounded();

    let mut event_logic = Box::pin(process_events::<A, E, C>(
        application,
        compositor,
        renderer,
        window,
        runtime,
        debug,
        receiver,
    ));

    let mut context =
        futures::task::Context::from_waker(futures::task::noop_waker_ref());

    event_loop.run(move |event, _, control_flow| {
        use winit::event_loop::ControlFlow;

        match event {
            winit::event::Event::WindowEvent { ref event, .. } => {
                handle_control_flow(event, control_flow);
            }
            _ => {
                *control_flow = ControlFlow::Wait;
            }
        }

        if let Some(event) = event.to_static() {
            sender.start_send(event).expect("Send event");

            if let Poll::Ready(_) = event_logic.as_mut().poll(&mut context) {
                panic!("Event logic has stopped running!");
            }
        }
    });
}

/// Runs an [`Application`] with an executor, compositor, and the provided
/// settings.
///
/// [`Application`]: trait.Application.html
async fn process_events<A, E, C>(
    mut application: A,
    mut compositor: C,
    mut renderer: A::Renderer,
    window: winit::window::Window,
    mut runtime: Runtime<E, Proxy<A::Message>, A::Message>,
    mut debug: Debug,
    mut receiver: mpsc::UnboundedReceiver<winit::event::Event<'_, A::Message>>,
) where
    A: Application + 'static,
    E: Executor + 'static,
    C: window::Compositor<Renderer = A::Renderer> + 'static,
{
    use iced_futures::futures::stream::StreamExt;
    use winit::event;

    let mut title = application.title();
    let mut mode = application.mode();
    let mut background_color = application.background_color();
    let mut scale_factor = application.scale_factor();

    let physical_size = window.inner_size();
    let mut viewport = Viewport::with_physical_size(
        Size::new(physical_size.width, physical_size.height),
        window.scale_factor() * scale_factor,
    );
    let mut resized = false;

    let surface = compositor.create_surface(&window);
    let mut swap_chain = compositor.create_swap_chain(
        &surface,
        physical_size.width,
        physical_size.height,
    );

    let clipboard = Clipboard::new(&window);
    // TODO: Encode cursor availability in the type-system
    let mut cursor_position = winit::dpi::PhysicalPosition::new(-1.0, -1.0);
    let mut mouse_interaction = mouse::Interaction::default();
    let mut modifiers = winit::event::ModifiersState::default();

    let mut user_interface = std::mem::ManuallyDrop::new(build_user_interface(
        &mut application,
        Cache::default(),
        &mut renderer,
        viewport.logical_size(),
        &mut debug,
    ));

    let mut primitive = user_interface.draw(
        &mut renderer,
        conversion::cursor_position(cursor_position, viewport.scale_factor()),
    );
    let mut events = Vec::new();
    let mut external_messages = Vec::new();

    debug.startup_finished();

    while let Some(event) = receiver.next().await {
        match event {
            event::Event::MainEventsCleared => {
                if events.is_empty() && external_messages.is_empty() {
                    continue;
                }

                debug.event_processing_started();
                let mut messages = user_interface.update(
                    &events,
                    conversion::cursor_position(
                        cursor_position,
                        viewport.scale_factor(),
                    ),
                    clipboard.as_ref().map(|c| c as _),
                    &mut renderer,
                );

                messages.extend(external_messages.drain(..));
                events.clear();
                debug.event_processing_finished();

                if messages.is_empty() {
                    debug.draw_started();
                    primitive = user_interface.draw(
                        &mut renderer,
                        conversion::cursor_position(
                            cursor_position,
                            viewport.scale_factor(),
                        ),
                    );
                    debug.draw_finished();
                } else {
                    let cache =
                        std::mem::ManuallyDrop::into_inner(user_interface)
                            .into_cache();

                    for message in messages.drain(..) {
                        debug.log_message(&message);

                        debug.update_started();
                        let command =
                            runtime.enter(|| application.update(message));
                        debug.update_finished();

                        runtime.spawn(command);
                    }

                    // Update subscriptions
                    let subscription = application.subscription();
                    runtime.track(subscription);

                    // Update window title
                    let new_title = application.title();

                    if title != new_title {
                        window.set_title(&new_title);

                        title = new_title;
                    }

                    // Update window mode
                    let new_mode = application.mode();

                    if mode != new_mode {
                        window.set_fullscreen(conversion::fullscreen(
                            window.current_monitor(),
                            new_mode,
                        ));

                        mode = new_mode;
                    }

                    // Update background color
                    background_color = application.background_color();

                    // Update scale factor
                    let new_scale_factor = application.scale_factor();

                    if scale_factor != new_scale_factor {
                        let size = window.inner_size();

                        viewport = Viewport::with_physical_size(
                            Size::new(size.width, size.height),
                            window.scale_factor() * new_scale_factor,
                        );

                        scale_factor = new_scale_factor;
                    }

                    user_interface =
                        std::mem::ManuallyDrop::new(build_user_interface(
                            &mut application,
                            cache,
                            &mut renderer,
                            viewport.logical_size(),
                            &mut debug,
                        ));

                    debug.draw_started();
                    primitive = user_interface.draw(
                        &mut renderer,
                        conversion::cursor_position(
                            cursor_position,
                            viewport.scale_factor(),
                        ),
                    );
                    debug.draw_finished();
                }

                window.request_redraw();
            }
            event::Event::UserEvent(message) => {
                external_messages.push(message);
            }
            event::Event::RedrawRequested(_) => {
                debug.render_started();

                if resized {
                    let physical_size = viewport.physical_size();

                    swap_chain = compositor.create_swap_chain(
                        &surface,
                        physical_size.width,
                        physical_size.height,
                    );

                    resized = false;
                }

                let new_mouse_interaction = compositor.draw(
                    &mut renderer,
                    &mut swap_chain,
                    &viewport,
                    background_color,
                    &primitive,
                    &debug.overlay(),
                );

                debug.render_finished();

                if new_mouse_interaction != mouse_interaction {
                    window.set_cursor_icon(conversion::mouse_interaction(
                        new_mouse_interaction,
                    ));

                    mouse_interaction = new_mouse_interaction;
                }

                // TODO: Handle animations!
                // Maybe we can use `ControlFlow::WaitUntil` for this.
            }
            event::Event::WindowEvent {
                event: window_event,
                ..
            } => {
                handle_window_event(
                    &window_event,
                    &window,
                    scale_factor,
                    &mut cursor_position,
                    &mut modifiers,
                    &mut viewport,
                    &mut resized,
                    &mut debug,
                );

                if let Some(event) = conversion::window_event(
                    &window_event,
                    viewport.scale_factor(),
                    modifiers,
                ) {
                    events.push(event.clone());
                    runtime.broadcast(event);
                }
            }
            _ => {}
        }
    }
}

/// Handles a `WindowEvent` and mutates the provided control flow to exit
/// if necessary.
pub fn handle_control_flow(
    event: &winit::event::WindowEvent<'_>,
    control_flow: &mut winit::event_loop::ControlFlow,
) {
    use winit::event::WindowEvent;
    use winit::event_loop::ControlFlow;

    match event {
        WindowEvent::CloseRequested => {
            *control_flow = ControlFlow::Exit;
        }
        #[cfg(target_os = "macos")]
        WindowEvent::KeyboardInput {
            input:
                winit::event::KeyboardInput {
                    virtual_keycode: Some(winit::event::VirtualKeyCode::Q),
                    state: winit::event::ElementState::Pressed,
                    ..
                },
            ..
        } if modifiers.logo() => {
            *control_flow = ControlFlow::Exit;
        }
        _ => {}
    }
}

/// Handles a `WindowEvent` and mutates the keyboard modifiers, viewport, and
/// resized flag accordingly.
pub fn handle_window_event(
    event: &winit::event::WindowEvent<'_>,
    window: &winit::window::Window,
    scale_factor: f64,
    cursor_position: &mut winit::dpi::PhysicalPosition<f64>,
    modifiers: &mut winit::event::ModifiersState,
    viewport: &mut Viewport,
    resized: &mut bool,
    _debug: &mut Debug,
) {
    use winit::event::WindowEvent;

    match event {
        WindowEvent::Resized(new_size) => {
            let size = Size::new(new_size.width, new_size.height);

            *viewport = Viewport::with_physical_size(
                size,
                window.scale_factor() * scale_factor,
            );
            *resized = true;
        }
        WindowEvent::ScaleFactorChanged {
            scale_factor: new_scale_factor,
            new_inner_size,
        } => {
            let size = Size::new(new_inner_size.width, new_inner_size.height);

            *viewport = Viewport::with_physical_size(
                size,
                new_scale_factor * scale_factor,
            );
            *resized = true;
        }
        WindowEvent::CursorMoved { position, .. } => {
            *cursor_position = *position;
        }
        WindowEvent::CursorLeft { .. } => {
            // TODO: Encode cursor availability in the type-system
            *cursor_position = winit::dpi::PhysicalPosition::new(-1.0, -1.0);
        }
        WindowEvent::ModifiersChanged(new_modifiers) => {
            *modifiers = *new_modifiers;
        }
        #[cfg(feature = "debug")]
        WindowEvent::KeyboardInput {
            input:
                winit::event::KeyboardInput {
                    virtual_keycode: Some(winit::event::VirtualKeyCode::F12),
                    state: winit::event::ElementState::Pressed,
                    ..
                },
            ..
        } => _debug.toggle(),
        _ => {}
    }
}

fn build_user_interface<'a, A: Application>(
    application: &'a mut A,
    cache: Cache,
    renderer: &mut A::Renderer,
    size: Size,
    debug: &mut Debug,
) -> UserInterface<'a, A::Message, A::Renderer> {
    debug.view_started();
    let view = application.view();
    debug.view_finished();

    debug.layout_started();
    let user_interface = UserInterface::build(view, size, cache, renderer);
    debug.layout_finished();

    user_interface
}
