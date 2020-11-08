//! Create interactive, native cross-platform applications.
mod state;

pub use state::State;

use crate::conversion;
use crate::mouse;
use crate::{
    Clipboard, Color, Command, Debug, Error, Executor, Mode, Proxy, Runtime,
    Settings, Size, Subscription,
};

use iced_futures::futures;
use iced_futures::futures::channel::mpsc;
use iced_graphics::window;
use iced_native::program::Program;
use iced_native::{Cache, UserInterface};

use std::mem::ManuallyDrop;

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
    use futures::task;
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

    let mut instance = Box::pin(run_instance::<A, E, C>(
        application,
        compositor,
        renderer,
        window,
        runtime,
        debug,
        receiver,
    ));

    let mut context = task::Context::from_waker(task::noop_waker_ref());

    event_loop.run(move |event, _, control_flow| {
        use winit::event_loop::ControlFlow;

        if let ControlFlow::Exit = control_flow {
            return;
        }

        if let Some(event) = event.to_static() {
            sender.start_send(event).expect("Send event");

            let poll = instance.as_mut().poll(&mut context);

            *control_flow = match poll {
                task::Poll::Pending => ControlFlow::Wait,
                task::Poll::Ready(_) => ControlFlow::Exit,
            };
        }
    });
}

async fn run_instance<A, E, C>(
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

    let surface = compositor.create_surface(&window);
    let clipboard = Clipboard::new(&window);

    let mut state = State::new(&application, &window);
    let mut viewport_version = state.viewport_version();
    let mut swap_chain = {
        let physical_size = state.physical_size();

        compositor.create_swap_chain(
            &surface,
            physical_size.width,
            physical_size.height,
        )
    };

    let mut user_interface = ManuallyDrop::new(build_user_interface(
        &mut application,
        Cache::default(),
        &mut renderer,
        state.logical_size(),
        &mut debug,
    ));

    let mut primitive =
        user_interface.draw(&mut renderer, state.cursor_position());
    let mut mouse_interaction = mouse::Interaction::default();

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
                    state.cursor_position(),
                    clipboard.as_ref().map(|c| c as _),
                    &mut renderer,
                );

                messages.extend(external_messages.drain(..));
                events.clear();
                debug.event_processing_finished();

                if !messages.is_empty() {
                    let cache =
                        ManuallyDrop::into_inner(user_interface).into_cache();

                    // Update application
                    update(
                        &mut application,
                        &mut runtime,
                        &mut debug,
                        messages,
                    );

                    // Update window
                    state.synchronize(&application, &window);

                    user_interface = ManuallyDrop::new(build_user_interface(
                        &mut application,
                        cache,
                        &mut renderer,
                        state.logical_size(),
                        &mut debug,
                    ));
                }

                debug.draw_started();
                primitive =
                    user_interface.draw(&mut renderer, state.cursor_position());
                debug.draw_finished();

                window.request_redraw();
            }
            event::Event::UserEvent(message) => {
                external_messages.push(message);
            }
            event::Event::RedrawRequested(_) => {
                debug.render_started();
                let current_viewport_version = state.viewport_version();

                if viewport_version != current_viewport_version {
                    let physical_size = state.physical_size();
                    let logical_size = state.logical_size();

                    debug.layout_started();
                    user_interface = ManuallyDrop::new(
                        ManuallyDrop::into_inner(user_interface)
                            .relayout(logical_size, &mut renderer),
                    );
                    debug.layout_finished();

                    debug.draw_started();
                    primitive = user_interface
                        .draw(&mut renderer, state.cursor_position());
                    debug.draw_finished();

                    swap_chain = compositor.create_swap_chain(
                        &surface,
                        physical_size.width,
                        physical_size.height,
                    );

                    viewport_version = current_viewport_version;
                }

                let new_mouse_interaction = compositor.draw(
                    &mut renderer,
                    &mut swap_chain,
                    state.viewport(),
                    state.background_color(),
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
                if requests_exit(&window_event, state.modifiers()) {
                    break;
                }

                state.update(&window, &window_event, &mut debug);

                if let Some(event) = conversion::window_event(
                    &window_event,
                    state.scale_factor(),
                    state.modifiers(),
                ) {
                    events.push(event.clone());
                    runtime.broadcast(event);
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
///
/// [`Application`]: trait.Application.html
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
/// [`Debug`] information accordingly.
///
/// [`UserInterface`]: struct.UserInterface.html
/// [`Application`]: trait.Application.html
/// [`Debug`]: struct.Debug.html
pub fn build_user_interface<'a, A: Application>(
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

/// Updates an [`Application`] by feeding it the provided messages, spawning any
/// resulting [`Command`], and tracking its [`Subscription`].
///
/// [`Application`]: trait.Application.html
/// [`Command`]: struct.Command.html
/// [`Subscription`]: struct.Subscription.html
pub fn update<A: Application, E: Executor>(
    application: &mut A,
    runtime: &mut Runtime<E, Proxy<A::Message>, A::Message>,
    debug: &mut Debug,
    messages: Vec<A::Message>,
) {
    for message in messages {
        debug.log_message(&message);

        debug.update_started();
        let command = runtime.enter(|| application.update(message));
        debug.update_finished();

        runtime.spawn(command);
    }

    let subscription = application.subscription();
    runtime.track(subscription);
}
