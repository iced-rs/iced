use futures::{FutureExt, stream::{LocalBoxStream, SelectAll, Peekable, unfold, StreamExt}};
use iced_native::{window::Backend, Event, trace::{Trace, Component::Setup}};
use super::{window::{self, Window}, Application};
use smithay_client_toolkit::{default_environment, init_default_environment, seat};

// Application state update
pub(crate) struct Update<'u, 'q, Item> {
    pub streams: &'u mut Peekable<SelectAll<LocalBoxStream<'q, Item>>>,
    pub events: &'u mut Vec<Event>,
}

// Shared across the application between user message channel, display interface events, keyboard repeat timer
pub(crate) enum Item<Message> {
    #[allow(dead_code)] Push(Message),
    Apply(std::io::Result<()>),
    KeyRepeat(u32),
    Quit,
}

pub(crate) type Streams<'q, M> = SelectAll<LocalBoxStream<'q, Item<M>>>;

use seat::pointer::ThemedPointer;
pub(crate) struct State<B:Backend> {
    pointer: Option<ThemedPointer>,
    keyboard: super::keyboard::Keyboard,
    pub window: Window<B>,
}

pub(crate) struct DispatchData<'d, 'u, 'q, 's, A:Application> {
    pub update: &'d mut Update<'u,'q, Item::<A::Message>>,
    pub state: &'s mut State<A::Backend>,
}

// wayland-client requires DispatchData:Any:'static (i.e erases lifetimes)
unsafe fn erase_lifetime<'d,'u,'q,'s,A:Application>(data: DispatchData<'d,'u,'q,'s,A>) -> DispatchData<'static,'static,'static,'static,A> {
    std::mem::transmute::<DispatchData::<'d,'u,'q,'s,A>, DispatchData::<'static,'static,'static,'static,A>>(data)
}
// todo: actualy restore lifetimes, not just allow whatever
unsafe fn restore_erased_lifetime<'d,'u,'q,'s,A:Application>(data: &mut DispatchData::<'static,'static,'static,'static,A>) -> &'d mut DispatchData::<'d,'u,'q,'s,A> {
    std::mem::transmute::<&mut DispatchData::<'static,'static,'static,'static,A>, &mut DispatchData::<'d,'u,'q,'s,A>>(data)
}

default_environment!(Env, desktop);

//pub async fn application<A:Application>(settings: Settings<A::Flags>, backend: <A::Backend as Backend>::Settings) -> Result<(),std::io::Error> {
pub fn application<A:Application+'static>(arguments: A::Flags, window: window::Settings, backend: <A::Backend as Backend>::Settings)
-> Result<impl core::future::Future<Output=Result<(),std::io::Error>>,std::io::Error> {
    let mut trace = Trace::new();
    let _ = trace.scope(Setup);

    /*let (runtime, mut application, receiver) = {
        use {futures::channel, super::sink_clone::SinkCloneExt, iced_native::{Executor, Runtime}};

        // Gathers runtime messages to update application (spawns a blocking thread)
        let (sink, receiver) = channel::mpsc::unbounded();
        mod sink_clone; // iced_futures::Runtime::Sender: Clone
        let mut runtime = Runtime::new(A::Executor::new().unwrap(), sink.with_(async move |x| Ok(Item::Push(x.await)))); // ?

        let (application, init_command) = runtime.enter(|| Application::new(arguments));
        runtime.spawn(init_command);
        runtime.track(application.subscription());

        application
    };*/
    let (mut application, _init_command) = A::new(arguments);

    let (env, _, queue) = init_default_environment!(Env, desktop).unwrap();

    mod nix {
        pub type RawPollFd = std::os::unix::io::RawFd;
        pub trait AsRawPollFd { fn as_raw_poll_fd(&self) -> RawPollFd; }
        impl AsRawPollFd for std::os::unix::io::RawFd { fn as_raw_poll_fd(&self) -> RawPollFd { *self } }
    }
    struct Async<T>(T);
    struct AsRawFd<T>(T);
    impl<T:nix::AsRawPollFd> std::os::unix::io::AsRawFd for AsRawFd<T> { fn as_raw_fd(&self) -> std::os::unix::io::RawFd { self.0.as_raw_poll_fd() /*->smol::Reactor*/ } }
    impl<T:nix::AsRawPollFd> Async<T> { fn new(io: T) -> Result<smol::Async<AsRawFd<T>>, std::io::Error> { smol::Async::new(AsRawFd(io)) } }
    impl nix::AsRawPollFd for &smithay_client_toolkit::reexports::client::EventQueue { fn as_raw_poll_fd(&self) -> nix::RawPollFd { self.display().get_connection_fd() } }

    let seat_handler = { // for a simple setup
        use seat::pointer::{ThemeManager, ThemeSpec};
        let theme_manager = ThemeManager::init(ThemeSpec::System, env.require_global(), env.require_global());
        env.listen_for_seats(move |seat, seat_data, mut data| {
            let DispatchData::<A>{state:State{pointer, .. }, ..} = unsafe{restore_erased_lifetime(data.get().unwrap())};
            if seat_data.has_pointer {
                assert!(pointer.is_none());
                *pointer = Some(theme_manager.theme_pointer_with_impl(&seat,
                    {
                        let mut pointer = crate::pointer::Pointer::default(); // Track focus and reconstruct scroll events
                        move/*pointer*/ |event, themed_pointer, mut data| {
                            let DispatchData::<A>{update: Update{events, ..}, state:State{window: Window{window, cursor, .. }, ..}} = data.get().unwrap();
                            pointer.handle(event, themed_pointer, events, window, cursor);
                        }
                    }
                ));
            }
            if seat_data.has_keyboard {
                seat.get_keyboard().quick_assign(|_, event, mut data| {
                    let DispatchData::<A>{update:Update{streams,events}, state} = data.get().unwrap();
                    events.extend( state.keyboard.map(streams.get_mut(), event).into_iter() );
                });
            }
        })
    };

    let mut state = State::<A::Backend> {
        pointer: None,
        keyboard: Default::default(),
        window: Window::new::<A>(env, window, backend),
    };

    Ok(async move /*queue*/ {
        let poll_queue = Async::new(&queue)?;  // Registers in the reactor
        let mut streams = SelectAll::new().peekable();
        //streams.push(receiver);
        streams.get_mut().push(
            unfold(poll_queue, async move |q| { // Apply message callbacks (&mut state)
                Some((Item::<A::Message>::Apply(q.with(
                    |q/*:&smithay_client_toolkit::reexports::client::EventQueue*/|
                        q.0.prepare_read().ok_or(std::io::Error::new(std::io::ErrorKind::Interrupted, "Dispatch all events before reading again"))?.read_events()
                    ).await
                ), q))
            }).boxed_local()
        );
        'run: loop {
            // Framing: Amortizes display update by pulling buffers as much as possible to the latest state before embarking on the intensive evaluation
            let mut messages = Vec::new();
            let mut events = Vec::new(); // Buffered event handling present the opportunity to reduce some redundant work (e.g resizes/motions+)
            while /*~poll_next*/ std::pin::Pin::new(&mut streams).peek().now_or_never().is_some() {
                let item = streams.next().now_or_never().unwrap();
                let item = item.ok_or(std::io::Error::new(std::io::ErrorKind::UnexpectedEof,""))?;
                match item {
                        Item::Push(message) => messages.push(message),
                        Item::Apply(_) => {
                            let mut update = Update{streams: &mut streams, events: &mut events};
                            let _ = queue.dispatch_pending(/*Any: 'static*/unsafe{&mut erase_lifetime(DispatchData::<A>{update: &mut update, state: &mut state})}, |_,_,_| ())?;
                        },
                        Item::KeyRepeat(key) => events.push( state.keyboard.key(key, true) ),
                        Item::Quit => break 'run,
                };
            }

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

            if state.window.update_size() || messages.len() > 0 || events.len() > 0 {
                let cursor = state.window.update(&mut application, messages, events, &mut trace); // Update state after gathering all pending events or/and on pending messages
                if state.window.cursor != cursor {
                    state.window.cursor = cursor;
                    for pointer in state.pointer.iter_mut() {
                        pointer.set_cursor(state.window.cursor, None).expect("Unknown cursor");
                    }
                }
            }
            queue.display().flush().unwrap();
            let _ = std::pin::Pin::new(&mut streams).peek().await;
        }
        drop(streams);
        drop(seat_handler);
        drop(queue);
        Ok(())
    })
}
