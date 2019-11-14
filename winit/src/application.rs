use crate::{
    conversion,
    input::{keyboard, mouse},
    renderer::{Target, Windowed},
    Cache, Container, Debug, Element, Event, Length, MouseCursor,
    UserInterface,
};

pub trait Application {
    type Renderer: Windowed;

    type Message: std::fmt::Debug;

    fn title(&self) -> String;

    fn update(&mut self, message: Self::Message);

    fn view(&mut self) -> Element<Self::Message, Self::Renderer>;

    fn run(mut self)
    where
        Self: 'static + Sized,
    {
        use winit::{
            event::{self, WindowEvent},
            event_loop::{ControlFlow, EventLoop},
            window::WindowBuilder,
        };

        let mut debug = Debug::new();
        let mut title = self.title();

        debug.startup_started();
        let event_loop = EventLoop::new();

        // TODO: Ask for window settings and configure this properly
        let window = WindowBuilder::new()
            .with_title(&title)
            .with_inner_size(winit::dpi::LogicalSize {
                width: 1280.0,
                height: 1024.0,
            })
            .build(&event_loop)
            .expect("Open window");

        let dpi = window.hidpi_factor();
        let mut size = window.inner_size();
        let mut new_size: Option<winit::dpi::LogicalSize> = None;

        let mut renderer = Self::Renderer::new();

        let mut target = {
            let (width, height) = to_physical(size, dpi);

            <Self::Renderer as Windowed>::Target::new(
                &window, width, height, dpi as f32, &renderer,
            )
        };

        debug.layout_started();
        let user_interface = UserInterface::build(
            document(&mut self, size, &mut debug),
            Cache::default(),
            &mut renderer,
        );
        debug.layout_finished();

        debug.draw_started();
        let mut primitive = user_interface.draw(&mut renderer);
        debug.draw_finished();

        let mut cache = Some(user_interface.into_cache());
        let mut events = Vec::new();
        let mut mouse_cursor = MouseCursor::OutOfBounds;
        debug.startup_finished();

        window.request_redraw();

        event_loop.run(move |event, _, control_flow| match event {
            event::Event::MainEventsCleared => {
                // TODO: We should be able to keep a user interface alive
                // between events once we remove state references.
                //
                // This will allow us to rebuild it only when a message is
                // handled.
                debug.layout_started();
                let mut user_interface = UserInterface::build(
                    document(&mut self, size, &mut debug),
                    cache.take().unwrap(),
                    &mut renderer,
                );
                debug.layout_finished();

                debug.event_processing_started();
                let messages =
                    user_interface.update(&renderer, events.drain(..));
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
                        log::debug!("Updating");

                        debug.log_message(&message);

                        debug.update_started();
                        self.update(message);
                        debug.update_finished();
                    }

                    // Update window title
                    let new_title = self.title();

                    if title != new_title {
                        window.set_title(&new_title);

                        title = new_title;
                    }

                    debug.layout_started();
                    let user_interface = UserInterface::build(
                        document(&mut self, size, &mut debug),
                        temp_cache,
                        &mut renderer,
                    );
                    debug.layout_finished();

                    debug.draw_started();
                    primitive = user_interface.draw(&mut renderer);
                    debug.draw_finished();

                    cache = Some(user_interface.into_cache());
                }

                window.request_redraw();
            }
            event::Event::RedrawRequested(_) => {
                debug.render_started();

                if let Some(new_size) = new_size.take() {
                    let dpi = window.hidpi_factor();
                    let (width, height) = to_physical(new_size, dpi);

                    target.resize(
                        width,
                        height,
                        window.hidpi_factor() as f32,
                        &renderer,
                    );

                    size = new_size;
                }

                let new_mouse_cursor =
                    renderer.draw(&primitive, &debug.overlay(), &mut target);

                debug.render_finished();

                if new_mouse_cursor != mouse_cursor {
                    window.set_cursor_icon(conversion::mouse_cursor(
                        new_mouse_cursor,
                    ));

                    mouse_cursor = new_mouse_cursor;
                }

                // TODO: Handle animations!
                // Maybe we can use `ControlFlow::WaitUntil` for this.
            }
            event::Event::WindowEvent {
                event: window_event,
                ..
            } => match window_event {
                WindowEvent::CursorMoved { position, .. } => {
                    events.push(Event::Mouse(mouse::Event::CursorMoved {
                        x: position.x as f32,
                        y: position.y as f32,
                    }));
                }
                WindowEvent::MouseInput { button, state, .. } => {
                    events.push(Event::Mouse(mouse::Event::Input {
                        button: conversion::mouse_button(button),
                        state: conversion::button_state(state),
                    }));
                }
                WindowEvent::MouseWheel { delta, .. } => match delta {
                    winit::event::MouseScrollDelta::LineDelta(
                        delta_x,
                        delta_y,
                    ) => {
                        events.push(Event::Mouse(
                            mouse::Event::WheelScrolled {
                                delta: mouse::ScrollDelta::Lines {
                                    x: delta_x,
                                    y: delta_y,
                                },
                            },
                        ));
                    }
                    winit::event::MouseScrollDelta::PixelDelta(position) => {
                        events.push(Event::Mouse(
                            mouse::Event::WheelScrolled {
                                delta: mouse::ScrollDelta::Pixels {
                                    x: position.x as f32,
                                    y: position.y as f32,
                                },
                            },
                        ));
                    }
                },
                WindowEvent::ReceivedCharacter(c) => {
                    events.push(Event::Keyboard(
                        keyboard::Event::CharacterReceived(c),
                    ));
                }
                WindowEvent::KeyboardInput {
                    input:
                        winit::event::KeyboardInput {
                            virtual_keycode: Some(virtual_keycode),
                            state,
                            ..
                        },
                    ..
                } => {
                    match (virtual_keycode, state) {
                        (
                            winit::event::VirtualKeyCode::F12,
                            winit::event::ElementState::Pressed,
                        ) => debug.toggle(),
                        _ => {}
                    }

                    events.push(Event::Keyboard(keyboard::Event::Input {
                        key_code: conversion::key_code(virtual_keycode),
                        state: conversion::button_state(state),
                    }));
                }
                WindowEvent::CloseRequested => {
                    *control_flow = ControlFlow::Exit;
                }
                WindowEvent::Resized(size) => {
                    new_size = Some(size.into());

                    log::debug!("Resized: {:?}", new_size);
                }
                _ => {}
            },
            _ => {
                *control_flow = ControlFlow::Wait;
            }
        })
    }
}

fn to_physical(size: winit::dpi::LogicalSize, dpi: f64) -> (u16, u16) {
    let physical_size = size.to_physical(dpi);

    (
        physical_size.width.round() as u16,
        physical_size.height.round() as u16,
    )
}

fn document<'a, Application>(
    application: &'a mut Application,
    size: winit::dpi::LogicalSize,
    debug: &mut Debug,
) -> Element<'a, Application::Message, Application::Renderer>
where
    Application: self::Application,
    Application::Message: 'static,
{
    debug.view_started();
    let view = application.view();
    debug.view_finished();

    Container::new(view)
        .width(Length::Units(size.width.round() as u16))
        .height(Length::Units(size.height.round() as u16))
        .into()
}
