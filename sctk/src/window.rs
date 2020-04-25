type Cursor = &'static str;
struct Window {
    pub window: SCTKWindow<ConceptFrame>, // surface -> refresh
    pub size: (u32, u32), pub scale_factor: u32, // surface -> window
    pub current_cursor: Cursor, // window -> pointer::Enter
}

impl Window {
    fn new() -> Self {
        let surface = env.create_surface_with_scale_callback(
            |scale, surface, mut state| {
                let DispatchData{state:State{ scale_factor, ..}} = state.get().unwrap();
                surface.set_buffer_scale(scale);
                *scale_factor = scale as u32;
            },
        );

        let window = {
            env.create_window::<ConceptFrame, _>(
                surface,
                settings.window.size,
                move |event, mut state| {
                    let DispatchData{
                        update: Update { events, .. },
                        state: State { window, size, resized, .. },
                    } = state.get().unwrap();
                    match event {
                        window::Event::Configure { new_size: None, .. } => (),
                        window::Event::Configure {
                            new_size: Some(new_size),
                            ..
                        } => {
                            *size = new_size;
                            events.push(Event::Window(crate::window::Event::Resized { width: new_size.0, height: new_size.1 }));
                        }
                        window::Event::Close => streams.push(stream::once(Item::Close)),
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

        let cache = None;

        use iced_native::window::Backend; // new
        let (mut backend, mut renderer) = Self::Backend::new(backend_settings);

        //let surface = crate::window_ext::NoHasRawWindowHandleBackend::create_surface(&mut backend, &/*window*/());
        // Initially display a lowres buffer, compositor will notify output mapping (surface enter) so we can immediately update with proper resolution
        let mut swap_chain = backend.create_swap_chain(&backend.create_surface(&surface), size.0, size.1);
        let mut cursor = crate::MouseCursor::OutOfBounds;

        Self{
            window,
            size, scale_factor: 1,
            current_cursor: "left_ptr",
        }
    }
    fn update(state: &mut State, messages: Vec<_>) -> Primitive {
        debug.profile(Event);
        for e in events.iter() {
            use crate::input::{*, keyboard::{Event::Input, KeyCode}};
            if let Event::Keyboard(key @ Input{state: ButtonState::Pressed, ..}) = e {
                match (key.modifiers_state, key.keycode) {
                    ModifiersState { logo: true, .. }, KeyCode::Q) => return false,
                    #[cfg(feature = "debug")] (_, KeyCode::F12) => debug.toggle(),
                    _ => (),
                }
            }
        }
        events.iter().cloned().for_each(|event| runtime.broadcast(event));

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
            cache = Some(user_interface.into_cache());
            // drop('user_interface &application)

            for message in messages {
                log::debug!("Updating");
                debug.log_message(&message);
                debug.profile(Update);
                runtime.spawn(runtime.enter(|| application.update(message)));
            }
            runtime.track(application.subscription());

            let new_title = application.title();
            if title != new_title {
                window.set_title(new_title.clone());
                title = new_title;
            }

            if mode != application.mode() {
                mode = application.mode();
                match mode {
                    Mode::Fullscreen => window.set_fullscreen(None),
                    Mode::Windowed => window.unset_fullscreen(),
                }
            }

            let user_interface = build_user_interface(
                &mut application,
                temp_cache,
                &mut renderer,
                *size,
                &mut debug,
            );

            debug.profile(Draw);
            primitive = user_interface.draw(&mut renderer);

            cache = Some(user_interface.into_cache());
        }
    }
    fn render(size: (u32, u32), scale_factor: u32) -> Cursor {
        debug.profile(Render);

        if self.size != size || self.scale_factor != scale_factor {
            swap_chain = backend.create_swap_chain(
                &surface,
                size.0 * *scale_factor,
                size.1 * *scale_factor,
            );
        }

        let cursor = backend.draw(
            &mut renderer,
            &mut swap_chain,
            &primitive,
            *scale_factor as f64,
            &debug.overlay(),
        );

        if self.cursor != cursor {
            self.cursor = cursor;
            for pointer in pointer.iter_mut() {
                pointer.set_cursor( crate::conversion::cursor(cursor), None).expect("Unknown cursor");
            }
        }
    }
}
