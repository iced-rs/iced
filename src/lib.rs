pub use iced_wgpu::{Primitive, Renderer};
pub use iced_winit::{
    button, slider, text, winit, Align, Button, Checkbox, Color, Image,
    Justify, Length, Radio, Slider, Text,
};

pub type Element<'a, Message> = iced_winit::Element<'a, Message, Renderer>;
pub type Row<'a, Message> = iced_winit::Row<'a, Message, Renderer>;
pub type Column<'a, Message> = iced_winit::Column<'a, Message, Renderer>;

pub trait UserInterface {
    type Message;

    fn update(&mut self, message: Self::Message);

    fn view(&mut self) -> Element<Self::Message>;

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

        let mut renderer = Renderer::new(&window);
        let mut target = renderer.target(width, height);

        let mut cache = Some(iced_winit::Cache::default());
        let mut events = Vec::new();
        let mut redraws = 0;
        let mut primitive = Primitive::None;

        window.request_redraw();

        event_loop.run(move |event, _, control_flow| match event {
            Event::EventsCleared => {
                // TODO: We should be able to keep a user interface alive
                // between events once we remove state references.
                //
                // This will allow us to rebuild it only when a message is
                // handled.
                let mut user_interface = iced_winit::UserInterface::build(
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

                    let user_interface = iced_winit::UserInterface::build(
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
                println!("Redrawing {}", redraws);
                renderer.draw(&mut target, &primitive);

                redraws += 1;

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

fn document<UserInterface>(
    user_interface: &mut UserInterface,
    width: u16,
    height: u16,
) -> Element<UserInterface::Message>
where
    UserInterface: self::UserInterface,
    UserInterface::Message: 'static,
{
    Column::new()
        .width(Length::Units(width))
        .height(Length::Units(height))
        .push(user_interface.view())
        .into()
}
