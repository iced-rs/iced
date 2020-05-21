use crate::{
    conversion, mouse, Clipboard, Command, Debug, Executor, Mode, Proxy,
    Runtime, Settings, Size, Subscription,
};
use iced_graphics::window;
use iced_graphics::Viewport;
use iced_native::program::{self, Program};

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
    /// [`Settings`]: struct.Settings.html
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
}

pub fn run<A, E, C>(
    settings: Settings<A::Flags>,
    compositor_settings: C::Settings,
) where
    A: Application + 'static,
    E: Executor + 'static,
    C: window::Compositor<Renderer = A::Renderer> + 'static,
{
    use winit::{
        event,
        event_loop::{ControlFlow, EventLoop},
    };

    let mut debug = Debug::new();
    debug.startup_started();

    let event_loop = EventLoop::with_user_event();
    let mut runtime = {
        let executor = E::new().expect("Create executor");
        let proxy = Proxy::new(event_loop.create_proxy());

        Runtime::new(executor, proxy)
    };

    let flags = settings.flags;
    let (application, init_command) = runtime.enter(|| A::new(flags));
    runtime.spawn(init_command);

    let subscription = application.subscription();
    runtime.track(subscription);

    let mut title = application.title();
    let mut mode = application.mode();

    let window = settings
        .window
        .into_builder(&title, mode, event_loop.primary_monitor())
        .build(&event_loop)
        .expect("Open window");

    let clipboard = Clipboard::new(&window);
    let mut mouse_interaction = mouse::Interaction::default();
    let mut modifiers = winit::event::ModifiersState::default();

    let physical_size = window.inner_size();
    let mut viewport = Viewport::with_physical_size(
        Size::new(physical_size.width, physical_size.height),
        window.scale_factor(),
    );
    let mut resized = false;

    let (mut compositor, mut renderer) = C::new(compositor_settings);

    let surface = compositor.create_surface(&window);

    let mut swap_chain = compositor.create_swap_chain(
        &surface,
        physical_size.width,
        physical_size.height,
    );

    let mut state = program::State::new(
        application,
        viewport.logical_size(),
        &mut renderer,
        &mut debug,
    );
    debug.startup_finished();

    event_loop.run(move |event, _, control_flow| match event {
        event::Event::MainEventsCleared => {
            let command = runtime.enter(|| {
                state.update(
                    clipboard.as_ref().map(|c| c as _),
                    viewport.logical_size(),
                    &mut renderer,
                    &mut debug,
                )
            });

            // If the application was updated
            if let Some(command) = command {
                runtime.spawn(command);

                let program = state.program();

                // Update subscriptions
                let subscription = program.subscription();
                runtime.track(subscription);

                // Update window title
                let new_title = program.title();

                if title != new_title {
                    window.set_title(&new_title);

                    title = new_title;
                }

                // Update window mode
                let new_mode = program.mode();

                if mode != new_mode {
                    window.set_fullscreen(conversion::fullscreen(
                        window.current_monitor(),
                        new_mode,
                    ));

                    mode = new_mode;
                }
            }

            window.request_redraw();
        }
        event::Event::UserEvent(message) => {
            state.queue_message(message);
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
                state.primitive(),
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
                control_flow,
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
                state.queue_event(event.clone());
                runtime.broadcast(event);
            }
        }
        _ => {
            *control_flow = ControlFlow::Wait;
        }
    })
}

pub fn handle_window_event(
    event: &winit::event::WindowEvent<'_>,
    window: &winit::window::Window,
    control_flow: &mut winit::event_loop::ControlFlow,
    modifiers: &mut winit::event::ModifiersState,
    viewport: &mut Viewport,
    resized: &mut bool,
    _debug: &mut Debug,
) {
    use winit::{event::WindowEvent, event_loop::ControlFlow};

    match event {
        WindowEvent::Resized(new_size) => {
            let size = Size::new(new_size.width, new_size.height);

            *viewport =
                Viewport::with_physical_size(size, window.scale_factor());
            *resized = true;
        }
        WindowEvent::CloseRequested => {
            *control_flow = ControlFlow::Exit;
        }
        WindowEvent::ModifiersChanged(new_modifiers) => {
            *modifiers = *new_modifiers;
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
