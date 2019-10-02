pub use iced_wgpu::Renderer;
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

        let mut renderer =
            Renderer::new(&window, size.width as u32, size.height as u32);

        let mut cache = Some(iced_winit::Cache::default());
        let mut events = Vec::new();

        window.request_redraw();

        event_loop.run(move |event, _, control_flow| match event {
            Event::EventsCleared => {
                // TODO: Once we remove lifetimes from widgets, we will be able
                // to keep user interfaces alive between events.
                //
                // This will allow us to only rebuild when a message is handled.
                let mut user_interface = iced_winit::UserInterface::build(
                    self.view(),
                    cache.take().unwrap(),
                    &mut renderer,
                );

                let messages = user_interface.update(events.drain(..));

                if messages.is_empty() {
                    let _ = user_interface.draw(&mut renderer);

                    cache = Some(user_interface.into_cache());
                } else {
                    let temp_cache = user_interface.into_cache();

                    for message in messages {
                        self.update(message);
                    }

                    let user_interface = iced_winit::UserInterface::build(
                        self.view(),
                        temp_cache,
                        &mut renderer,
                    );

                    let _ = user_interface.draw(&mut renderer);

                    cache = Some(user_interface.into_cache());
                };

                window.request_redraw();
            }
            Event::WindowEvent {
                event: WindowEvent::RedrawRequested,
                ..
            } => {
                println!("Redrawing");
                renderer.draw();
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
