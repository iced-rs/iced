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

        let size = window.inner_size().to_physical(window.hidpi_factor());;

        let mut renderer =
            Renderer::new(&window, size.width as u32, size.height as u32);

        window.request_redraw();

        event_loop.run(move |event, _, control_flow| match event {
            Event::EventsCleared => {
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
