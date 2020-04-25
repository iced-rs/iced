use iced_native::{window, Cache, Command, Element, Executor, Subscription, UserInterface};
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

    fn future(
        settings: Settings<Self::Flags>,
        backend_settings: <Self::Backend as window::Backend>::Settings,
    ) -> impl futures::Future<Output=()> {
        use {iced_native::{Runtime, Event, debug::{Debug, Component::{Startup}}}, futures::{stream::{self, Stream, SelectAll}, channel}, super::{Item, Update}};

        let mut debug = Debug::new();
        debug.profile(Startup);

        let streams = SelectAll::new().peekable();

        let (runtime, application) = {
            // Gathers runtime messages to update application (spawns a blocking thread)
            let (sink, receiver) = channel::mpsc::unbounded();
            let mut runtime = Runtime::new(Self::Executor::new().unwrap(), sink.with_(async move |x| Item::Push(x.await)));
            streams.push(receiver);

            let (mut application, init_command) = runtime.enter(|| Self::new(settings.flags));
            runtime.spawn(init_command);
            runtime.track(application.subscription());
            application
        };

        use smithay_client_toolkit::{default_environment, init_default_environment, seat, window::{Window as SCTKWindow, ConceptFrame, Decorations}};
        default_environment!(Env, desktop);
        let (env, display, queue) = init_default_environment!(Env, desktop).unwrap();

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

        struct State {
            keyboard: super::keyboard::Keyboard,
            pointer: Option<super::pointer::Pointer>,
            window: super::window::Window,
        }
        //
        struct DispatchData<'t, St:Stream+Unpin> {
            update: &'t mut Update<'t, St>,
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
                                let DispatchData{update, state:State{ window, current_cursor, .. }} = data.get().unwrap();
                                pointer.handle(event, themed_pointer, update, window, current_cursor);
                            }
                        }
                    ).unwrap());
                }
                if seat_data.has_keyboard {
                    let _ = map_keyboard(&seat, None, RepeatKind::System,
                        |event, _, data| {
                            let DispatchData{update, state} = data.get().unwrap();
                            state.keyboard.handle(event, update);
                        }
                    ).unwrap();
                }
            });
        };

        let mut state = State {
            keyboard: Default::default(),
            pointer: None,
            window: Window::new(),
        };

        async {
            loop {
                // Framing: Amortizes display update by pulling buffers as much as possible to the latest state before embarking on the intensive evaluation
                let messages = Vec::new();
                let events = Vec::new(); // Buffered event handling present the opportunity to reduce some redundant work (e.g resizes/motions+)
                while let Some(item) = streams.peek().now_or_never() {
                    let mut update = Update{streams, events};
                    match item {
                        Item::Push(message) => messages.push(message),
                        Item::Apply => queue.dispatch_pending(&mut DispatchData{state, update}, |_,_,_| ()),
                        Item::KeyRepeat(event) => state.keyboard.handle(update, event),
                        Item::Close => return;
                    }
                    let _next = streams.next(); // That should just drop the peek
                    assert!(_next.now_or_never().is_some());
                }
                if events.len() > 0 || messages.len() > 0 {
                    update(&mut state, messages, events); // Update state after gathering all pending events or/and on pending messages
                if *need_refresh {
            *need_refresh = false;
        }

                queue.get_ref().display().flush().unwrap();
                smol::block_on(streams.peek());
            }
            drop(seat_handler);
            drop(display);
        }
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
    ) {
        smol::run(Self::future(settings, backend_settings));
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
