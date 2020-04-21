use crate::{
    Debug, Executor, Command, Subscription, Settings,
    window, Element, Mode,
    UserInterface, Cache
};

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
    type Backend: window::Backend+crate::window_ext::NoHasRawWindowHandleBackend;

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
    ) -> Element<'_, Self::Message, <Self::Backend as crate::window::Backend>::Renderer>;

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
        use {
            futures::{channel::mpsc::unbounded, select_biased},
            smithay_client_toolkit::{
                reexports::client::EventQueue,
                default_environment, init_default_environment,
                reexports::client::protocol::wl_pointer as pointer,
                seat::{keyboard::{self, RepeatKind}, pointer::{AutoThemer, ThemeSpec, AutoPointer}},
                window::{self, Window, ConceptFrame, Decorations},
            },
            crate::{
                Runtime,
                Event, input::{self, keyboard::{KeyCode, ModifiersState}, ButtonState, mouse},
                MouseCursor, //Clipboard,
                conversion,
            },
        };

        let mut debug = Debug::new();

        debug.startup_started();

        let (sink, channel) = unbounded();
        let mut runtime = Runtime::new(Self::Executor::new().unwrap(), sink);

        let flags = settings.flags;
        let (mut application, init_command) = runtime.enter(|| Self::new(flags));
        runtime.spawn(init_command);

        let subscription = application.subscription();
        runtime.track(subscription);

        default_environment!(Env, desktop);
        let (env, display, queue) = init_default_environment!(Env, desktop).unwrap();

        let auto_themer = AutoThemer::init(ThemeSpec::System, env.require_global(), env.require_global());

        enum ControlFlow { Wait, Exit, }
        use futures::stream::StreamExt;
        use async_std::stream::{Interval as NoFuseInterval, interval};
        //#[derive(derive_more::Deref)] struct Interval(IntervalNoFused)
        //impl FusedStream for Interval { fn is_terminated() { return false; } }
        type Interval = futures::stream::Fuse<NoFuseInterval>;
        struct Repeat {key: KeyCode, utf8: Option<String>, interval: Interval}
        struct State {
            control_flow: ControlFlow,
            events: Vec<Event>,
            window: Window<ConceptFrame>,
            scale_factor: u32,
            size: (u32, u32),
            resized: bool,
            need_refresh: bool,
            current_cursor: &'static str,
            pointer: Option<AutoPointer>,
            modifiers: ModifiersState,
            repeat: Option<Repeat>,
        }

        let seat_listener = env.listen_for_seats(move |seat, seat_data, mut state| {
            let State{ pointer, .. } = state.get().unwrap();
                if seat_data.has_pointer {
                    let mut mouse_focus = None;
                    let mut axis_buffer = None;
                    let mut axis_discrete_buffer = None;
                    *pointer = Some(auto_themer.theme_pointer_with_impl(&seat, move |event, pointer, mut state| {
                        let State{window, current_cursor, events, .. } = state.get().unwrap();
                        match event {
                            pointer::Event::Enter { surface, surface_x:x,surface_y:y, .. } if surface == *window.surface() => {
                                mouse_focus = Some(surface);
                                pointer.set_cursor(current_cursor, None).expect("Unknown cursor");
                                events.push(Event::Mouse(mouse::Event::CursorEntered));
                                events.push(Event::Mouse(mouse::Event::CursorMoved{x: x as f32, y: y as f32}));
                            }
                            pointer::Event::Leave { .. } => {
                                mouse_focus = None;
                                events.push(Event::Mouse(mouse::Event::CursorEntered));
                            }
                            pointer::Event::Motion { surface_x: x, surface_y: y, .. } if mouse_focus.is_some() => {
                                events.push(Event::Mouse(mouse::Event::CursorMoved{x: x as f32, y: y as f32}));
                            }
                            pointer::Event::Button { button, state, .. } if mouse_focus.is_some() => {
                                let state = match state {
                                    pointer::ButtonState::Pressed => ButtonState::Pressed,
                                    pointer::ButtonState::Released => ButtonState::Released,
                                    _ => unreachable!(),
                                };
                                events.push(Event::Mouse(mouse::Event::Input{button: conversion::button(button), state}));
                            }
                            pointer::Event::Axis { axis, value, .. } if mouse_focus.is_some() => {
                                let (mut x, mut y) = axis_buffer.unwrap_or((0.0, 0.0));
                                match axis {
                                    // wayland vertical sign convention is the inverse of winit
                                    pointer::Axis::VerticalScroll => y -= value as f32,
                                    pointer::Axis::HorizontalScroll => x += value as f32,
                                    _ => unreachable!(),
                                }
                                axis_buffer = Some((x, y));
                            }
                            pointer::Event::Frame if mouse_focus.is_some() => {
                                let axis_buffer = axis_buffer.take();
                                let axis_discrete_buffer = axis_discrete_buffer.take();
                                if let Some((x, y)) = axis_discrete_buffer {
                                    events.push(Event::Mouse(mouse::Event::WheelScrolled {delta: mouse::ScrollDelta::Lines {x: x as f32, y: y as f32}}));
                                }
                                else if let Some((x, y)) = axis_buffer {
                                    events.push(Event::Mouse(mouse::Event::WheelScrolled { delta: mouse::ScrollDelta::Pixels {x,y}}));
                                }
                            }
                            pointer::Event::AxisSource { .. } => (),
                            pointer::Event::AxisStop { .. } => (),
                            pointer::Event::AxisDiscrete { axis, discrete } if mouse_focus.is_some() => {
                                let (mut x, mut y) = axis_discrete_buffer.unwrap_or((0, 0));
                                match axis {
                                    // wayland vertical sign convention is the inverse of iced
                                    pointer::Axis::VerticalScroll => y -= discrete,
                                    pointer::Axis::HorizontalScroll => x += discrete,
                                    _ => unreachable!(),
                                }
                                axis_discrete_buffer = Some((x, y));
                            }
                            _ => unreachable!(),
                    }
                }));
            }
            if seat_data.has_keyboard {
                let (_, _) = keyboard::map_keyboard(&seat, None, RepeatKind::System, move |event, _, mut state| {
                    let State{ modifiers, repeat, events, .. } = state.get().unwrap();
                    match event {
                        keyboard::Event::Enter { .. } => (),
                        keyboard::Event::Leave { .. } => *repeat = None,
                        keyboard::Event::Key {
                            rawkey,
                            keysym,
                            state,
                            utf8,
                            ..
                        } => {
                            let key = conversion::key(rawkey, keysym);
                            events.push(Event::Keyboard(input::keyboard::Event::Input{
                                key_code: key,
                                state: if state == keyboard::KeyState::Pressed { ButtonState::Pressed } else { ButtonState::Released },
                                modifiers: *modifiers,
                            }));
                            if let Some(ref txt) = utf8 {
                                for char in txt.chars() {
                                    events.push(Event::Keyboard(input::keyboard::Event::CharacterReceived(char)));
                                }
                            }
                            if state == keyboard::KeyState::Pressed {
                                *repeat = Some(Repeat{key, utf8, interval: interval(std::time::Duration::from_millis(100)).fuse()})
                            } else {
                                if let Some(Repeat{key:repeat_key, ..}) = repeat { if *repeat_key==key { *repeat = None } }
                            }
                        }
                        keyboard::Event::Modifiers {
                            modifiers: new_modifiers,
                        } => {
                            *modifiers = conversion::modifiers(new_modifiers);
                        }
                        keyboard::Event::Repeat {..} => (), // todo: xkb only, no sctk repeat
                    }
                }).unwrap();
            }
        });

        let surface = env.create_surface_with_scale_callback( |scale, surface, mut state| {
            let State{ scale_factor, need_refresh, .. } = state.get().unwrap();
            surface.set_buffer_scale(scale);
            *scale_factor = scale as u32;
            *need_refresh = true;
        });

        let window = env.create_window::<ConceptFrame, _>(surface, settings.window.size, move |event, mut state| {
            let State{
                window,
                ref mut size,
                ref mut resized,
                events,
                control_flow,
                ..
            } = state.get().unwrap();
            match event {
                window::Event::Configure{new_size:None, ..} => (),
                window::Event::Configure{new_size:Some(new_size), ..} => {
                    *size = new_size;
                    *resized = true;
                    events.push(Event::Window(crate::window::Event::Resized{
                        width: new_size.0,
                        height: new_size.1,
                    }));
                }
                window::Event::Close => *control_flow = ControlFlow::Exit,
                window::Event::Refresh => window.refresh(),
            }
        }).unwrap();

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
        let mut state = State{ // for event callbacks
            window,
            scale_factor: 1,
            size,
            resized: false,
            need_refresh: false,
            pointer: None,
            current_cursor: "left_ptr",
            modifiers: Default::default(),
            repeat: None,
            events: Vec::new(),
            control_flow: ControlFlow::Wait,
        };

        let mut update = {
            use iced_native::window::Backend; // new
            let (mut backend, mut renderer) = Self::Backend::new(backend_settings);

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

            let surface = crate::window_ext::NoHasRawWindowHandleBackend::create_surface(&mut backend, &/*window*/());
            let mut swap_chain = {
                // Initially display a lowres buffer, compositor will notify output mapping (surface enter) so we can immediately update with proper resolution
                backend.create_swap_chain(
                    &surface,
                    size.0, size.1,
                )
            };

            let mut mouse_cursor = MouseCursor::OutOfBounds; // for idle callback
            move |state:&mut State, messages:Vec<_>| {
                let State{ window, scale_factor, size, resized, need_refresh, pointer, events, control_flow, .. } = state;

                if events.is_empty() && messages.is_empty() {
                    return;
                }

                for e in events.iter() {
                    if let Event::Keyboard(input::keyboard::Event::Input{state:ButtonState::Pressed, modifiers, key_code}) = e {
                        match (modifiers, key_code) {
                            (ModifiersState{logo:true, ..}, KeyCode::Q) => *control_flow = ControlFlow::Exit,
                            #[cfg(feature = "debug")] (_, KeyCode::F12) => debug.toggle(),
                            _ => (),
                        }
                    }
                }

                debug.event_processing_started();
                events.iter().cloned().for_each(|event| runtime.broadcast(event) );

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

                if messages.is_empty() { // Redraw after any event processing even yielding no messages
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
                            size.0* *scale_factor,
                            size.1* *scale_factor,
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
                        //pointer.as_mut().map(|pointer|
                        for pointer in pointer.iter_mut() {
                            pointer.set_cursor(conversion::mouse_cursor(new_mouse_cursor), None).expect("Unknown cursor");
                        }

                        mouse_cursor = new_mouse_cursor;
                    }

                    *need_refresh = false;
                }
            }
        };

        fn repeat(state: &mut State) {
            if let State{ modifiers, repeat:Some(Repeat{key, utf8, ..}), events, .. } = state {
                events.push(Event::Keyboard(input::keyboard::Event::Input {
                    key_code: *key,
                    state: ButtonState::Pressed,
                    modifiers: *modifiers,
                }));
                if let Some(txt) = utf8 {
                    for char in txt.chars() {
                        events.push(Event::Keyboard(input::keyboard::Event::CharacterReceived(char)));
                    }
                }
            } else { unreachable!(); }
        }

        fn dispatch<State:'static>(queue: &mut EventQueue, state: &mut State) {
            loop {
                if let Some(guard) = queue.prepare_read() {
                    guard.read_events() .unwrap_or_else(|e| assert!(e.kind() != std::io::ErrorKind::WouldBlock) );
                }
                if queue.dispatch_pending(state, |_, _, _| { unimplemented!(); }).unwrap() == 0 { break; }
            }
        }

        let mut display = {
            let fd = std::convert::identity(display).get_connection_fd(); // get_connection_fd: only poll
            use std::os::unix::io::FromRawFd;
            unsafe { async_std::os::unix::net::UnixStream::from_raw_fd(fd) } // from_raw_fd: hidden ownership transfer
        };
        let mut queue = queue;
        let mut channel = channel;
        while let ControlFlow::Wait = state.control_flow {
            let mut messages = Vec::new();
            //use futures::{future::FutureExt/*fuse*/, stream::StreamExt, io::AsyncReadExt}, async_std::stream::from_fn};
            use futures::{future::{OptionFuture, FutureExt}, io::AsyncReadExt};
            //let interval_next = None; //OptionFuture<_>
            //_  = state.repeat.map(|r| r.interval.fuse().select_next_some()).into() => repeat(&mut state),
            //if let Some(Repeat{interval}) = state.repeat {
                //interval_next = Some(r.interval.fuse().select_next_some());
            //}
            select_biased! {
                m = channel.select_next_some() => messages.push(m),
                _ = display.read(&mut[]).fuse() => dispatch(&mut queue, &mut state), // Triggers per event callbacks which mutate state
                _  = state.repeat.as_mut().map(|r| r.interval.select_next_some()).into():OptionFuture<_> => repeat(&mut state),
                //_  = state.repeat.as_mut().map(|r| r.interval.fuse().select_next_some()).into():OptionFuture<_> => repeat(&mut state),
                //_  = state.repeat.as_mut().map(|ref mut r| r.interval.fuse().select_next_some()).into():OptionFuture<_> => repeat(&mut state),
                //_ = from_fn(|| state.repeat.as_mut().map(|r| &mut r.interval).map(|i| i.next() ) ).fuse().select_next_some() => repeat(&mut state),
                //_ = from_fn(|| state.repeat.as_mut().map(|r| &mut r.interval).map(|i| i.next() ) ).fuse().select_next_some() => repeat(&mut state),
                complete => break,
                default => {
                    update(&mut state, messages.drain(..).collect()); // Update state after dispatching all pending events or/and on pending messages
                    queue.display().flush().unwrap();
                }
            }
        }
        drop(seat_listener)
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
        iced_native::Size::new(
            size.0 as f32,
            size.1 as f32,
        ),
        cache,
        renderer,
    );
    debug.layout_finished();

    user_interface
}
