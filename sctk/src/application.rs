use iced_native::{window, Cache, Command, Debug, Element, Executor, Subscription, UserInterface};
use crate::{Mode, Settings};

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
    type Backend: window::Backend
        + crate::window_ext::NoHasRawWindowHandleBackend;

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
    fn subscription(&self) -> Subscription<Self::Message>;

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
        <Self::Backend as crate::window::Backend>::Renderer,
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
        backend_settings: <Self::Backend as window::Backend>::Settings,
    ) where
        Self: 'static,
    {
        use {iced_native::{Event, Runtime}, futures::{stream::{self, Stream, SelectAll}, channel}, super::{Frame, ControlFlow}};

        let mut debug = Debug::new();
        debug.startup_started();

        // Shared between user message channel, display interface events, keyboard repeat timer
        enum Item<Message> {
            Message(Message),
            Events,
            KeyRepeat(crate::keyboard::Event<'static>),
        }

        let (sink, channel) = channel::mpsc::unbounded();
        let mut runtime = Runtime::new(Self::Executor::new().unwrap(), sink.with_(async move |x| Item::Message(x.await)));

        let flags = settings.flags;
        let (mut application, init_command) =
            runtime.enter(|| Self::new(flags));
        runtime.spawn(init_command);

        let subscription = application.subscription();
        runtime.track(subscription);

        use smithay_client_toolkit::{default_environment, init_default_environment, seat, window::{Window, ConceptFrame, Decorations}};
        default_environment!(Env, desktop);
        let (env, display, queue) = init_default_environment!(Env, desktop).unwrap();

        /// Mutable state time shared by stream handlers on main thread
        struct State {
            keyboard: crate::keyboard::Keyboard,
            pointer: Option<crate::pointer::Pointer>,

            window: Window<ConceptFrame>,
            current_cursor: &'static str,
            scale_factor: u32,
            size: (u32, u32),
            resized: bool,
            need_refresh: bool,
        }
        //
        struct DispatchData<'t, St:Stream+Unpin> {
            frame: &'t mut Frame<'t, St>,
            state: &'t mut State,
        }

        let seat_handler = { // for a simple setup
            use seat::{
                pointer::{ThemeManager, ThemeSpec},
                keyboard::{map_keyboard, RepeatKind},
            };

            let theme_manager = ThemeManager::init(
                ThemeSpec::System,
                env.require_global(),
                env.require_global(),
            );

            env.listen_for_seats(move |seat, seat_data, mut data| {
                let DispatchData{state:State{pointer, .. }} = data.get().unwrap();
                if seat_data.has_pointer {
                    assert!(pointer.is_none());
                    *pointer = Some(theme_manager.theme_pointer_with_impl(&seat,
                        {
                            let pointer = crate::pointer::Pointer::default(); // Track focus and reconstruct scroll events
                            move/*pointer*/ |event, themed_pointer, data| {
                                let DispatchData{frame, state:State{ window, current_cursor, .. }} = data.get().unwrap();
                                pointer.handle(event, themed_pointer, frame, window, current_cursor);
                            }
                        }
                    ).unwrap());
                }
                if seat_data.has_keyboard {
                    let _ = map_keyboard(&seat, None, RepeatKind::System,
                        |event, _, data| {
                            let DispatchData{frame, state} = data.get().unwrap();
                            state.keyboard.handle(event, frame);
                        }
                    ).unwrap();
                }
            });
        };

        let surface = env.create_surface_with_scale_callback(
            |scale, surface, mut state| {
                let DispatchData{state:State {
                    scale_factor,
                    need_refresh,
                    ..
                }} = state.get().unwrap();
                surface.set_buffer_scale(scale);
                *scale_factor = scale as u32;
                *need_refresh = true;
            },
        );

        let window = {
            env.create_window::<ConceptFrame, _>(
                surface,
                settings.window.size,
                move |event, mut state| {
                    let DispatchData{
                        frame: Frame { control_flow, events, .. },
                        state: State { window, size, resized, .. },
                    } = state.get().unwrap();
                    match event {
                        window::Event::Configure { new_size: None, .. } => (),
                        window::Event::Configure {
                            new_size: Some(new_size),
                            ..
                        } => {
                            *size = new_size;
                            *resized = true;
                            events.push(Event::Window(
                                crate::window::Event::Resized {
                                    width: new_size.0,
                                    height: new_size.1,
                                },
                            ));
                        }
                        window::Event::Close => {
                            *control_flow = ControlFlow::Exit
                        }
                        window::Event::Refresh => window.refresh(),
                    }
                },
            ).unwrap()
        };

        let mut title = application.title();
        window.set_title(title.clone());
        window.set_resizable(settings.window.resizable);
        window.set_decorate(if settings.window.decorations {
            Decorations::FollowServer
        } else {
            Decorations::None
        });
        let mut mode = application.mode();
        if let Mode::Fullscreen = mode {
            window.set_fullscreen(None);
        }

        let size = settings.window.size; // in pixels / scale factor (initially 1) (i.e initially both 'physical'/'logical')
        let mut state = State {
            // for event callbacks
            window,
            scale_factor: 1,
            size,
            resized: false,
            need_refresh: false,
            pointer: None,
            current_cursor: "left_ptr",
            events: Vec::new(),
            control_flow: ControlFlow::Wait,
        };

        let mut update = {
            use iced_native::window::Backend; // new
            let (mut backend, mut renderer) =
                Self::Backend::new(backend_settings);

            let user_interface = build_user_interface(
                &mut application,
                Cache::default(),
                &mut renderer,
                size,
                &mut debug,
            );

            debug.draw_started();
            let mut primitive = user_interface.draw(&mut renderer);
            debug.draw_finished();

            let mut cache = Some(user_interface.into_cache());

            display.flush().unwrap();
            debug.startup_finished();

            let surface =
                crate::window_ext::NoHasRawWindowHandleBackend::create_surface(
                    &mut backend,
                    &/*window*/(),
                );
            let mut swap_chain = {
                // Initially display a lowres buffer, compositor will notify output mapping (surface enter) so we can immediately update with proper resolution
                backend.create_swap_chain(&surface, size.0, size.1)
            };

            let mut mouse_cursor = crate::MouseCursor::OutOfBounds; // for idle callback
            move |state: &mut State, messages: Vec<_>| {
                let State {
                    window,
                    scale_factor,
                    size,
                    resized,
                    need_refresh,
                    pointer,
                    events,
                    control_flow,
                    ..
                } = state;

                if events.is_empty() && messages.is_empty() {
                    return;
                }

                for e in events.iter() {
                    use crate::input::{*, keyboard::*};
                    if let Event::Keyboard(keyboard::Event::Input {
                        state: ButtonState::Pressed,
                        modifiers,
                        key_code,
                    }) = e
                    {
                        match (modifiers, key_code) {
                            (ModifiersState { logo: true, .. }, KeyCode::Q) => {
                                *control_flow = ControlFlow::Exit
                            }
                            #[cfg(feature = "debug")]
                            (_, KeyCode::F12) => debug.toggle(),
                            _ => (),
                        }
                    }
                }

                debug.event_processing_started();
                events
                    .iter()
                    .cloned()
                    .for_each(|event| runtime.broadcast(event));

                let mut user_interface = build_user_interface(
                    &mut application,
                    cache.take().unwrap(),
                    &mut renderer,
                    *size,
                    &mut debug,
                );

                let messages = {
                    // Deferred on_event(event, &mut messages) so user_interface mut borrows (application, renderer, debug)
                    let mut sync_messages = user_interface.update(
                        events.drain(..),
                        None, /*clipboard
                              .as_ref()
                              .map(|c| c as &dyn iced_native::Clipboard),*/
                        &renderer,
                    );
                    sync_messages.extend(messages);
                    sync_messages
                };
                debug.event_processing_finished();

                if messages.is_empty() {
                    // Redraw after any event processing even yielding no messages
                    debug.draw_started();
                    primitive = user_interface.draw(&mut renderer);
                    debug.draw_finished();

                    cache = Some(user_interface.into_cache());
                } else {
                    // When there are messages, we are forced to rebuild twice
                    // for now :^). Why ?
                    let temp_cache = user_interface.into_cache();

                    for message in messages {
                        log::debug!("Updating");

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
                        window.set_title(new_title.clone());
                        title = new_title;
                    }

                    // Update window mode
                    let new_mode = application.mode();

                    if mode != new_mode {
                        match new_mode {
                            Mode::Fullscreen => {
                                window.set_fullscreen(None);
                            }
                            Mode::Windowed => {
                                window.unset_fullscreen();
                            }
                        }
                        mode = new_mode;
                    }

                    let user_interface = build_user_interface(
                        &mut application,
                        temp_cache,
                        &mut renderer,
                        *size,
                        &mut debug,
                    );

                    debug.draw_started();
                    primitive = user_interface.draw(&mut renderer);
                    debug.draw_finished();

                    cache = Some(user_interface.into_cache());
                }

                *need_refresh = true;
                if *need_refresh {
                    debug.render_started();

                    if *resized {
                        swap_chain = backend.create_swap_chain(
                            &surface,
                            size.0 * *scale_factor,
                            size.1 * *scale_factor,
                        );

                        *resized = false;
                    }

                    let new_mouse_cursor = backend.draw(
                        &mut renderer,
                        &mut swap_chain,
                        &primitive,
                        *scale_factor as f64,
                        &debug.overlay(),
                    );

                    debug.render_finished();

                    if new_mouse_cursor != mouse_cursor {
                        for pointer in pointer.iter_mut() {
                            pointer.set_cursor( crate::conversion::mouse_cursor(new_mouse_cursor), None).expect("Unknown cursor");
                        }
                        mouse_cursor = new_mouse_cursor;
                    }

                    *need_refresh = false;
                }
            }
        };

        let streams = SelectAll::new().peekable();  //<Item>;
        // Gather pending messages (spawns a blocking thread)
        streams.push(channel);

        // Dispatch socket to per event callbacks which mutate state
        mod nix {
            pub struct RawPollFd(pub std::os::unix::io::RawFd);
            pub trait AsRawPollFd {
                fn as_raw_poll_fd(&self) -> RawPollFd;
            }
            impl AsRawPollFd for std::os::unix::io::RawFd { fn as_raw_poll_fd(&self) -> RawPollFd { RawPollFd(self.as_raw_fd().0) } }
        }
        struct Async<T>(T);
        impl<T> Async<T> {
            fn new(poll_fd: impl nix::AsRawPollFd) -> Result<smol::Async<T>, std::io::Error> {
                struct AsRawFd<T>(T);
                impl<T> std::os::unix::io::AsRawFd for AsRawFd<T> { fn as_raw_fd(&self) -> std::os::unix::io::RawFd { self.as_raw_poll_fd().0 /*->smol::Reactor*/ } }
                smol::Async::new()
            }
        }
        impl nix::AsRawPollFd for smithay_client_toolkit::reexports::client::EventQueue {
            fn as_raw_poll_fd(&self) -> nix::RawPollFd { nix::RawPollFd(self.display().get_connection_fd()) }
        }

        let queue = Async::new(queue).unwrap();  // Registers in the reactor
        streams.push( stream::poll_fn(|_| queue.with_mut(|q| Item::Event(q.prepare_read()?.read_events()?) ) ) );

        smol::run(async {
            while let ControlFlow::Wait = state.control_flow {
                queue.get_ref().display().flush().unwrap();
                // Framing: Amortizes display update by pulling buffers as much as possible to the latest state before embarking on the intensive evaluation
                let messages = Vec::new();
                let events = Vec::new(); // Buffered event handling present the opportunity to reduce some redundant work (e.g resizes/motions+)
                while let Some(item) = streams.peek().now_or_never() {
                    let mut frame = Frame{streams, events};
                    match item {
                        Item::Message(message) => messages.push(message),
                        Item::Events => queue.dispatch_pending(&mut DispatchData{state, frame}, |_,_,_| ()),
                        Item::KeyRepeat(event) => state.keyboard.handle(frame, event),
                    }
                    let _next = streams.next(); // That should just drop the peek
                    assert!(_next.now_or_never().is_some());
                }
                update(&mut state, messages); // Update state after dispatching all pending events or/and on pending messages
                queue.get_ref().display().flush().unwrap();
                smol::block_on(streams.peek());
            }
        });
        drop(seat_handler);
        drop(display)
    }
}

fn build_user_interface<'a, A: Application>(
    application: &'a mut A,
    cache: Cache,
    renderer: &mut <A::Backend as window::Backend>::Renderer,
    size: (u32, u32), // logical
    debug: &mut Debug,
) -> UserInterface<'a, A::Message, <A::Backend as window::Backend>::Renderer> {
    debug.view_started();
    let view = application.view();
    debug.view_finished();

    debug.layout_started();
    let user_interface = UserInterface::build(
        view,
        iced_native::Size::new(size.0 as f32, size.1 as f32),
        cache,
        renderer,
    );
    debug.layout_finished();

    user_interface
}
