use iced_native::{window::Backend, Executor};
use super::{Settings, Application};

pub fn future<Application: super::Application>(settings: Settings<Application::Flags>, backend_settings:
    <<Application as super::Application>::Backend as Backend>::Settings) // fixme
    -> impl futures::Future<Output=()> {
        use iced_native::{Runtime, Event, trace::{Trace, Component::{Setup}}};
        use futures::{stream::{self, Stream, StreamExt, SelectAll}, channel};
        use super::{Item, Update, Keyboard, Window};

        let mut trace = Trace::new();
        trace.scope(Setup);

        let streams = SelectAll::new().peekable();

        let (runtime, application) = {
            // Gathers runtime messages to update application (spawns a blocking thread)
            let (sink, receiver) = channel::mpsc::unbounded();
            use super::sink_clone::SinkCloneExt;
            let mut runtime = Runtime::new(Application::Executor::new().unwrap(), sink.with_(async move |x| Item::Push(x.await)));
            streams.push(receiver);

            let (mut application, init_command) = runtime.enter(|| Application::new(settings.flags));
            runtime.spawn(init_command);
            runtime.track(application.subscription());
            application
        };

        use smithay_client_toolkit::{default_environment, init_default_environment, seat, window};
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

        struct State<Window> {
            keyboard: Keyboard,
            window: Window,
        }
        //
        struct DispatchData<'t, St:Stream+Unpin, W> {
            update: &'t mut Update<'t, St>,
            state: &'t mut State<W>,
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

        let surface = env.create_surface_with_scale_callback(
            |scale, surface, mut state| {
                let DispatchData{state:State{window, ..}} = state.get().unwrap();
                surface.set_buffer_scale(scale);
                *window.scale_factor = scale as u32;
            }
        );

        let window = env.create_window::<window::ConceptFrame, _>(surface, settings.window.size,
            move |event, mut state| {
                let DispatchData{update: Update { events, .. }, state: State{window}} = state.get().unwrap();
                use window::Event::*;
                match event {
                    Configure { new_size: None, .. } => (),
                    Configure { new_size: Some(new_size), .. } => {
                        *window.size = new_size;
                        events.push(Event::Window(iced_native::Event::Resized { width: new_size.0, height: new_size.1 }));
                    }
                    Close => streams.push(stream::once(super::Item::Close)),
                    Refresh => window.refresh(),
                }
            }
        ).unwrap();

        let mut state = State {
            keyboard: Default::default(),
            pointer: None,
            window: Window::new(window, settings.window.size),
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
                        Item::Close => return,
                    }
                    let _next = streams.next(); // That should just drop the peek
                    assert!(_next.now_or_never().is_some());
                }

                //debug.profile(Event);
                for e in events.iter() {
                    use crate::input::{*, keyboard::{Event::Input, KeyCode, ModifiersState}};
                    if let Event::Keyboard(key @ Input{state: ButtonState::Pressed, ..}) = e {
                        match (key.modifiers_state, key.keycode) {
                            (ModifiersState { logo: true, .. }, KeyCode::Q) => return false,
                            #[cfg(feature = "debug")] (_, KeyCode::F12) => debug.toggle(),
                            _ => (),
                        }
                    }
                }
                events.iter().cloned().for_each(|event| runtime.broadcast(event));

                if [
                    if events.len() > 0 || messages.len() > 0 {
                        window.update(&mut state, messages, events); // Update state after gathering all pending events or/and on pending messages
                        true
                    } else { false },
                    if window.buffer_size != window.size || window.buffer_scale_factor != window.scale_factor {
                        (window.buffer_size, window.buffer_scale_factor) = (window.size, window.scale_factor);
                        window.swap_chain = window.backend.create_swap_chain(&surface,
                            window.buffer_size.0 * *window.buffer_scale_factor,
                            window.buffer_size.1 * *window.buffer_scale_factor);
                        true
                    } else { false },
                ].contains(true) {
                    window.render();
                }
                queue.get_ref().display().flush().unwrap();
                smol::block_on(streams.peek());
            }
            drop(seat_handler);
            drop(display);
        }
    }
