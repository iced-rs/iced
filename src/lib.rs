pub use iced_wgpu::{Primitive, Renderer};
pub use iced_winit::{
    button, slider, text, winit, Align, Background, Checkbox, Color, Image,
    Justify, Length, Radio, Slider, Text,
};

pub type Element<'a, Message> = iced_winit::Element<'a, Message, Renderer>;
pub type Row<'a, Message> = iced_winit::Row<'a, Message, Renderer>;
pub type Column<'a, Message> = iced_winit::Column<'a, Message, Renderer>;
pub type Button<'a, Message> = iced_winit::Button<'a, Message, Renderer>;

pub trait Application {
    type Message;

    fn update(&mut self, message: Self::Message);

    fn view(&mut self) -> Element<Self::Message>;

    fn run(self)
    where
        Self: 'static + Sized,
    {
        iced_winit::Application::run(Instance(self))
    }
}

struct Instance<A: Application>(A);

impl<A> iced_winit::Application for Instance<A>
where
    A: Application,
{
    type Renderer = Renderer;
    type Message = A::Message;

    fn update(&mut self, message: Self::Message) {
        self.0.update(message);
    }

    fn view(&mut self) -> Element<Self::Message> {
        self.0.view()
    }
}
