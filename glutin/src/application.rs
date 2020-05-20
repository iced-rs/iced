use crate::{
    mouse, Cache, Command, Element, Executor, Runtime, Size, Subscription,
    UserInterface,
};
use iced_graphics::window;
use iced_graphics::Viewport;
use iced_winit::conversion;
use iced_winit::{Clipboard, Debug, Mode, Proxy, Settings};

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
pub trait Application: Sized {
    /// The graphics backend to use to draw the [`Application`].
    ///
    /// [`Application`]: trait.Application.html
    type Compositor: window::GLCompositor;

    /// The [`Executor`] that will run commands and subscriptions.
    ///
    /// [`Executor`]: trait.Executor.html
    type Executor: Executor;

    /// The type of __messages__ your [`Application`] will produce.
    ///
    /// [`Application`]: trait.Application.html
    type Message: std::fmt::Debug + Send;

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

    /// Handles a __message__ and updates the state of the [`Application`].
    ///
    /// This is where you define your __update logic__. All the __messages__,
    /// produced by either user interactions or commands, will be handled by
    /// this method.
    ///
    /// Any [`Command`] returned will be executed immediately in the background.
    ///
    /// [`Application`]: trait.Application.html
    /// [`Command`]: struct.Command.html
    fn update(&mut self, message: Self::Message) -> Command<Self::Message>;

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

    /// Returns the widgets to display in the [`Application`].
    ///
    /// These widgets can produce __messages__ based on user interaction.
    ///
    /// [`Application`]: trait.Application.html
    fn view(
        &mut self,
    ) -> Element<
        '_,
        Self::Message,
        <Self::Compositor as window::GLCompositor>::Renderer,
    >;

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

    /// Runs the [`Application`] with the provided [`Settings`].
    ///
    /// On native platforms, this method will take control of the current thread
    /// and __will NOT return__.
    ///
    /// It should probably be that last thing you call in your `main` function.
    ///
    /// [`Application`]: trait.Application.html
    /// [`Settings`]: struct.Settings.html
    fn run(
        settings: Settings<Self::Flags>,
        backend_settings: <Self::Compositor as window::GLCompositor>::Settings,
    ) where
        Self: 'static,
    {
        use glutin::{
            event::{self, WindowEvent},
            event_loop::{ControlFlow, EventLoop},
            ContextBuilder,
        };
        use iced_graphics::window::GLCompositor as _;

        let mut debug = Debug::new();

        debug.startup_started();
        let event_loop = EventLoop::with_user_event();
        let mut external_messages = Vec::new();

        let mut runtime = {
            let executor = Self::Executor::new().expect("Create executor");

            Runtime::new(executor, Proxy::new(event_loop.create_proxy()))
        };

        let flags = settings.flags;
        let (mut application, init_command) =
            runtime.enter(|| Self::new(flags));
        runtime.spawn(init_command);

        let subscription = application.subscription();
        runtime.track(subscription);

        let mut title = application.title();
        let mut mode = application.mode();

        let context = {
            let window_builder = settings.window.into_builder(
                &title,
                mode,
                event_loop.primary_monitor(),
            );

            let context = ContextBuilder::new()
                .with_vsync(true)
                .build_windowed(window_builder, &event_loop)
                .expect("Open window");

            #[allow(unsafe_code)]
            unsafe {
                context.make_current().expect("Make OpenGL context current")
            }
        };

        let physical_size = context.window().inner_size();
        let mut viewport = Viewport::with_physical_size(
            Size::new(physical_size.width, physical_size.height),
            context.window().scale_factor(),
        );
        let mut resized = false;

        let clipboard = Clipboard::new(&context.window());

        #[allow(unsafe_code)]
        let (mut compositor, mut renderer) = unsafe {
            Self::Compositor::new(backend_settings, |address| {
                context.get_proc_address(address)
            })
        };

        let user_interface = build_user_interface(
            &mut application,
            Cache::default(),
            &mut renderer,
            viewport.logical_size(),
            &mut debug,
        );

        debug.draw_started();
        let mut primitive = user_interface.draw(&mut renderer);
        debug.draw_finished();

        let mut cache = Some(user_interface.into_cache());
        let mut events = Vec::new();
        let mut mouse_interaction = mouse::Interaction::default();
        let mut modifiers = glutin::event::ModifiersState::default();
        debug.startup_finished();

        context.window().request_redraw();

        event_loop.run(move |event, _, control_flow| match event {
            event::Event::MainEventsCleared => {
                if events.is_empty() && external_messages.is_empty() {
                    return;
                }

                let mut user_interface = build_user_interface(
                    &mut application,
                    cache.take().unwrap(),
                    &mut renderer,
                    viewport.logical_size(),
                    &mut debug,
                );

                debug.event_processing_started();
                events
                    .iter()
                    .cloned()
                    .for_each(|event| runtime.broadcast(event));

                let mut messages = user_interface.update(
                    events.drain(..),
                    clipboard
                        .as_ref()
                        .map(|c| c as &dyn iced_native::Clipboard),
                    &renderer,
                );
                messages.extend(external_messages.drain(..));
                debug.event_processing_finished();

                if messages.is_empty() {
                    debug.draw_started();
                    primitive = user_interface.draw(&mut renderer);
                    debug.draw_finished();

                    cache = Some(user_interface.into_cache());
                } else {
                    // When there are messages, we are forced to rebuild twice
                    // for now :^)
                    let temp_cache = user_interface.into_cache();

                    for message in messages {
                        debug.log_message(&message);

                        debug.update_started();
                        let command =
                            runtime.enter(|| application.update(message));
                        runtime.spawn(command);
                        debug.update_finished();
                    }

                    let subscription = application.subscription();
                    runtime.track(subscription);

                    // Update window title
                    let new_title = application.title();

                    if title != new_title {
                        context.window().set_title(&new_title);

                        title = new_title;
                    }

                    // Update window mode
                    let new_mode = application.mode();

                    if mode != new_mode {
                        context.window().set_fullscreen(
                            conversion::fullscreen(
                                context.window().current_monitor(),
                                new_mode,
                            ),
                        );

                        mode = new_mode;
                    }

                    let user_interface = build_user_interface(
                        &mut application,
                        temp_cache,
                        &mut renderer,
                        viewport.logical_size(),
                        &mut debug,
                    );

                    debug.draw_started();
                    primitive = user_interface.draw(&mut renderer);
                    debug.draw_finished();

                    cache = Some(user_interface.into_cache());
                }

                context.window().request_redraw();
            }
            event::Event::UserEvent(message) => {
                external_messages.push(message);
            }
            event::Event::RedrawRequested(_) => {
                debug.render_started();

                if resized {
                    let physical_size = viewport.physical_size();

                    context.resize(glutin::dpi::PhysicalSize {
                        width: physical_size.width,
                        height: physical_size.height,
                    });
                    compositor.resize_viewport(physical_size);

                    resized = false;
                }

                let new_mouse_interaction = compositor.draw(
                    &mut renderer,
                    &viewport,
                    &primitive,
                    &debug.overlay(),
                );

                context.swap_buffers().expect("Swap buffers");
                debug.render_finished();

                if new_mouse_interaction != mouse_interaction {
                    context.window().set_cursor_icon(
                        conversion::mouse_interaction(new_mouse_interaction),
                    );

                    mouse_interaction = new_mouse_interaction;
                }

                // TODO: Handle animations!
                // Maybe we can use `ControlFlow::WaitUntil` for this.
            }
            event::Event::WindowEvent {
                event: window_event,
                ..
            } => {
                match window_event {
                    WindowEvent::Resized(new_size) => {
                        let size = Size::new(new_size.width, new_size.height);

                        viewport = Viewport::with_physical_size(
                            size,
                            context.window().scale_factor(),
                        );
                        resized = true;
                    }
                    WindowEvent::CloseRequested => {
                        *control_flow = ControlFlow::Exit;
                    }
                    WindowEvent::ModifiersChanged(new_modifiers) => {
                        modifiers = new_modifiers;
                    }
                    #[cfg(target_os = "macos")]
                    WindowEvent::KeyboardInput {
                        input:
                            glutin::event::KeyboardInput {
                                virtual_keycode:
                                    Some(glutin::event::VirtualKeyCode::Q),
                                state: glutin::event::ElementState::Pressed,
                                ..
                            },
                        ..
                    } if modifiers.logo() => {
                        *control_flow = ControlFlow::Exit;
                    }
                    #[cfg(feature = "debug")]
                    WindowEvent::KeyboardInput {
                        input:
                            glutin::event::KeyboardInput {
                                virtual_keycode:
                                    Some(glutin::event::VirtualKeyCode::F12),
                                state: glutin::event::ElementState::Pressed,
                                ..
                            },
                        ..
                    } => debug.toggle(),
                    _ => {}
                }

                if let Some(event) = conversion::window_event(
                    &window_event,
                    viewport.scale_factor(),
                    modifiers,
                ) {
                    events.push(event);
                }
            }
            _ => {
                *control_flow = ControlFlow::Wait;
            }
        })
    }
}

fn build_user_interface<'a, A: Application>(
    application: &'a mut A,
    cache: Cache,
    renderer: &mut <A::Compositor as window::GLCompositor>::Renderer,
    size: Size,
    debug: &mut Debug,
) -> UserInterface<
    'a,
    A::Message,
    <A::Compositor as window::GLCompositor>::Renderer,
> {
    debug.view_started();
    let view = application.view();
    debug.view_finished();

    debug.layout_started();
    let user_interface = UserInterface::build(view, size, cache, renderer);
    debug.layout_finished();

    user_interface
}
