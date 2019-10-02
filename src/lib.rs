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

    fn run(self)
    where
        Self: Sized,
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

        let renderer = Renderer::new(&window);

        event_loop.run(move |event, _, control_flow| match event {
            Event::EventsCleared => {
                window.request_redraw();
            }
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                *control_flow = ControlFlow::Exit;
            }
            _ => {
                *control_flow = ControlFlow::Poll;
            }
        })
    }
}
