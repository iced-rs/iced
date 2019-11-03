use crate::{
    column, conversion, input::mouse, renderer::Windowed, Cache, Column,
    Element, Event, Length, MouseCursor, UserInterface,
};

pub trait Application {
    type Renderer: Windowed + column::Renderer;

    type Message;

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

        let event_loop = EventLoop::new();

        // TODO: Ask for window settings and configure this properly
        let window = WindowBuilder::new()
            .with_inner_size(winit::dpi::LogicalSize {
                width: 1280.0,
                height: 1024.0,
            })
            .build(&event_loop)
            .expect("Open window");

        let mut size: Size = window
            .inner_size()
            .to_physical(window.hidpi_factor())
            .into();
        let mut new_size: Option<Size> = None;

        let mut renderer = Self::Renderer::new(&window);
        let mut target = renderer.target(size.width, size.height);

        let user_interface = UserInterface::build(
            document(&mut self, size),
            Cache::default(),
            &renderer,
        );

        let mut primitive = user_interface.draw(&mut renderer);
        let mut cache = Some(user_interface.into_cache());
        let mut events = Vec::new();
        let mut mouse_cursor = MouseCursor::OutOfBounds;

        window.request_redraw();

        event_loop.run(move |event, _, control_flow| match event {
            event::Event::MainEventsCleared => {
                // TODO: We should be able to keep a user interface alive
                // between events once we remove state references.
                //
                // This will allow us to rebuild it only when a message is
                // handled.
                let mut user_interface = UserInterface::build(
                    document(&mut self, size),
                    cache.take().unwrap(),
                    &renderer,
                );

                let messages =
                    user_interface.update(&renderer, events.drain(..));

                if messages.is_empty() {
                    primitive = user_interface.draw(&mut renderer);

                    cache = Some(user_interface.into_cache());
                } else {
                    // When there are messages, we are forced to rebuild twice
                    // for now :^)
                    let temp_cache = user_interface.into_cache();

                    for message in messages {
                        log::debug!("Updating");

                        self.update(message);
                    }

                    let user_interface = UserInterface::build(
                        document(&mut self, size),
                        temp_cache,
                        &renderer,
                    );

                    primitive = user_interface.draw(&mut renderer);

                    cache = Some(user_interface.into_cache());
                }

                window.request_redraw();
            }
            event::Event::RedrawRequested(_) => {
                if let Some(new_size) = new_size.take() {
                    target = renderer.target(new_size.width, new_size.height);
                    size = new_size;
                }

                let new_mouse_cursor = renderer.draw(&primitive, &mut target);

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
                    // TODO: Remove when renderer supports HiDPI
                    let physical_position =
                        position.to_physical(window.hidpi_factor());

                    events.push(Event::Mouse(mouse::Event::CursorMoved {
                        x: physical_position.x as f32,
                        y: physical_position.y as f32,
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
                        // TODO: Remove when renderer supports HiDPI
                        let physical_position =
                            position.to_physical(window.hidpi_factor());

                        events.push(Event::Mouse(
                            mouse::Event::WheelScrolled {
                                delta: mouse::ScrollDelta::Pixels {
                                    x: physical_position.x as f32,
                                    y: physical_position.y as f32,
                                },
                            },
                        ));
                    }
                },
                WindowEvent::CloseRequested => {
                    *control_flow = ControlFlow::Exit;
                }
                WindowEvent::Resized(size) => {
                    new_size =
                        Some(size.to_physical(window.hidpi_factor()).into());

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

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
struct Size {
    width: u16,
    height: u16,
}

impl From<winit::dpi::PhysicalSize> for Size {
    fn from(physical_size: winit::dpi::PhysicalSize) -> Self {
        Self {
            width: physical_size.width.round() as u16,
            height: physical_size.height.round() as u16,
        }
    }
}

fn document<Application>(
    application: &mut Application,
    size: Size,
) -> Element<Application::Message, Application::Renderer>
where
    Application: self::Application,
    Application::Message: 'static,
{
    Column::new()
        .width(Length::Units(size.width))
        .height(Length::Units(size.height))
        .push(application.view())
        .into()
}
