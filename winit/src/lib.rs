pub use iced_native::*;
pub use winit;

pub mod conversion;

pub use iced_native::renderer::Windowed;

pub trait Application {
    type Renderer: iced_native::renderer::Windowed
        + iced_native::column::Renderer;

    type Message;

    fn update(&mut self, message: Self::Message);

    fn view(&mut self) -> Element<Self::Message, Self::Renderer>;

    fn run(mut self)
    where
        Self: 'static + Sized,
    {
        use winit::{
            event::{Event, WindowEvent},
            event_loop::{ControlFlow, EventLoop},
            window::WindowBuilder,
        };

        let event_loop = EventLoop::new();

        // TODO: Ask for window settings and configure this properly
        let window = WindowBuilder::new()
            .build(&event_loop)
            .expect("Open window");

        let size = window.inner_size().to_physical(window.hidpi_factor());;
        let (width, height) = (size.width as u16, size.height as u16);

        let mut renderer = Self::Renderer::new(&window);
        let mut target = renderer.target(width, height);

        let user_interface = UserInterface::build(
            document(&mut self, width, height),
            Cache::default(),
            &mut renderer,
        );

        let mut primitive = user_interface.draw(&mut renderer);
        let mut cache = Some(user_interface.into_cache());
        let mut events = Vec::new();

        window.request_redraw();

        event_loop.run(move |event, _, control_flow| match event {
            Event::EventsCleared => {
                // TODO: We should be able to keep a user interface alive
                // between events once we remove state references.
                //
                // This will allow us to rebuild it only when a message is
                // handled.
                let mut user_interface = UserInterface::build(
                    document(&mut self, width, height),
                    cache.take().unwrap(),
                    &mut renderer,
                );

                let messages = user_interface.update(events.drain(..));

                if messages.is_empty() {
                    primitive = user_interface.draw(&mut renderer);

                    cache = Some(user_interface.into_cache());
                } else {
                    // When there are messages, we are forced to rebuild twice
                    // for now :^)
                    let temp_cache = user_interface.into_cache();

                    for message in messages {
                        self.update(message);
                    }

                    let user_interface = UserInterface::build(
                        document(&mut self, width, height),
                        temp_cache,
                        &mut renderer,
                    );

                    primitive = user_interface.draw(&mut renderer);

                    cache = Some(user_interface.into_cache());
                }

                window.request_redraw();
            }
            Event::WindowEvent {
                event: WindowEvent::RedrawRequested,
                ..
            } => {
                renderer.draw(&mut target, &primitive);

                // TODO: Handle animations!
                // Maybe we can use `ControlFlow::WaitUntil` for this.
            }
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                *control_flow = ControlFlow::Exit;
            }
            _ => {
                *control_flow = ControlFlow::Wait;
            }
        })
    }
}

fn document<Application>(
    application: &mut Application,
    width: u16,
    height: u16,
) -> Element<Application::Message, Application::Renderer>
where
    Application: self::Application,
    Application::Message: 'static,
{
    Column::new()
        .width(Length::Units(width))
        .height(Length::Units(height))
        .push(application.view())
        .into()
}
