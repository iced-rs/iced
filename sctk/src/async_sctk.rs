use iced_native::{window::Backend, Executor};
use super::{Settings, Window, Application};

pub async fn application<A:Application>(settings: Settings<A::Flags>, backend: <A::Backend as Backend>::Settings) -> Result<(),std::io::Error> {
    use {std::iter::once, futures::{pin_mut, FutureExt, stream::{unfold, iter, StreamExt, SelectAll}}};
    use iced_native::{Event, trace::{Trace, Component::{Setup}}};
    use super::{Item, Update, Keyboard};

    let mut trace = Trace::new();
    trace.scope(Setup);

    let mut streams = SelectAll::new().peekable();

    /*let (runtime, application) = {
        use {futures::channel, super::sink_clone::SinkCloneExt, iced_native::Runtime};

        // Gathers runtime messages to update application (spawns a blocking thread)
        let (sink, receiver) = channel::mpsc::unbounded();
        let mut runtime = Runtime::new(A::Executor::new().unwrap(), sink.with_(async move |x| Ok(Item::Push(x.await))));
        streams.push(receiver);

        let (mut application, init_command) = runtime.enter(|| Application::new(settings.flags));
        runtime.spawn(init_command);
        runtime.track(application.subscription());

        application
    };*/
    let application = A::new(settings.flags);

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
    streams.get_mut().push(
        unfold(queue, async move |mut queue| {
            Some((Item::<A::Message>::Apply(queue.with_mut(
                |q:&mut smithay_client_toolkit::reexports::client::EventQueue|
                    q.prepare_read().ok_or(std::io::Error::new(std::io::ErrorKind::Interrupted, "Dispatch all events before reading again"))?.read_events()
                ).await
            ), queue))
        }).boxed_local()
    );

    struct State<B:Backend> {
        keyboard: Keyboard,
        window: Window<B>,
    }
    //
    struct DispatchData<'t, Item, B:Backend> {
        update: &'t mut Update<'t, Item>,
        state: &'t mut State<B>,
    }

    let seat_handler = { // for a simple setup
        use seat::{pointer::{ThemeManager, ThemeSpec}, keyboard::map_keyboard};
        let theme_manager = ThemeManager::init(ThemeSpec::System, env.require_global(), env.require_global());
        env.listen_for_seats(move |seat, seat_data, mut data| {
            let DispatchData::<Item::<A::Message>,_>{state:State{window:Window::<A::Backend>{pointer,..}, .. }, ..} = data.get().unwrap();
            if seat_data.has_pointer {
                assert!(pointer.is_none());
                *pointer = Some(theme_manager.theme_pointer_with_impl(&seat,
                    {
                        let mut pointer = crate::pointer::Pointer::default(); // Track focus and reconstruct scroll events
                        move/*pointer*/ |event, themed_pointer, mut data| {
                            let DispatchData::<Item::<A::Message>,_>{update: Update{events, ..}, state:State{window: Window::<A::Backend>{window, current_cursor, .. }, ..}} = data.get().unwrap();
                            pointer.handle(event, themed_pointer, events, window, current_cursor);
                        }
                    }
                ));
            }
            if seat_data.has_keyboard {
                let _ = map_keyboard(&seat, None,
                    |event, _, mut data| {
                        let DispatchData::<Item::<A::Message>,A::Backend>{update, state} = data.get().unwrap();
                        state.keyboard.handle(update, &event);
                    }
                ).unwrap();
            }
        });
    };

    let surface = env.create_surface_with_scale_callback(
        |scale, surface, mut state| {
            let DispatchData::<Item::<A::Message>,_>{state:State::<A::Backend>{window, ..}, ..} = state.get().unwrap();
            surface.set_buffer_scale(scale);
            window.scale_factor = scale as u32;
        }
    );

    let window = env.create_window::<window::ConceptFrame, _>(surface, settings.window.size,
        move |event, mut state| {
            let DispatchData::<Item::<A::Message>,_>{update: Update{streams, events, .. }, state: State::<A::Backend>{window, ..}} = state.get().unwrap();
            use window::Event::*;
            match event {
                Configure { new_size: None, .. } => (),
                Configure { new_size: Some(new_size), .. } => {
                    window.size = new_size;
                    events.push(Event::Window(iced_native::window::Event::Resized {width: new_size.0, height: new_size.1}));
                }
                Close => streams.get_mut().push(iter(once(super::Item::Close)).boxed()),
                Refresh => window.window.refresh(),
            }
        }
    ).unwrap();

    let mut state = State::<A::Backend> {
        keyboard: Default::default(),
        window: Window::new(window, settings.window, backend),
    };

    loop {
        // Framing: Amortizes display update by pulling buffers as much as possible to the latest state before embarking on the intensive evaluation
        let messages = Vec::new();
        let events = Vec::new(); // Buffered event handling present the opportunity to reduce some redundant work (e.g resizes/motions+)
        loop {
            let item = {
                pin_mut!(streams);
                if let Some(item) = streams.peek().now_or_never() { item } else { break; }
            };
            let item = item.ok_or(std::io::Error::new(std::io::ErrorKind::UnexpectedEof,""))?;
            {
                let mut update = Update{streams: &mut streams, events: &mut events};
                match item {
                    Item::Push(message) => messages.push(message),
                    Item::Apply(_) => { queue.get_mut().dispatch_pending(&mut DispatchData{update: &mut update, state: &mut state}, |_,_,_| ())?; }
                    Item::KeyRepeat(event) => state.keyboard.handle(&mut update, event),
                    Item::Close => return Ok(()),
                }
            }
            let _next = streams.next(); // That should just drop the peek
            assert!(_next.now_or_never().is_some());
        }

        //debug.profile(Event);
        for e in events.iter() {
            use crate::input::{*, keyboard::{Event::Input, KeyCode, ModifiersState}};
            if let Event::Keyboard(Input{state: ButtonState::Pressed, modifiers, key_code, ..}) = e {
                match (modifiers, key_code) {
                    (ModifiersState { logo: true, .. }, KeyCode::Q) => Err(std::io::Error::new(std::io::ErrorKind::Other,"User force quit with Logo+Q"))?,
                    #[cfg(feature = "debug")] (_, KeyCode::F12) => debug.toggle(),
                    _ => (),
                }
            }
        }
        //events.iter().cloned().for_each(|event| runtime.broadcast(event));
        state.window.update(messages, events); // Update state after gathering all pending events or/and on pending messages
        display.flush().unwrap();
        pin_mut!(streams);
        streams.peek().await;
    }
    /*drop(seat_handler);
    drop(queue);
    drop(display);*/
}
