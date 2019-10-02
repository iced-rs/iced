pub use iced_wgpu::Renderer;
pub use iced_winit::{
    button, slider, text, Align, Button, Checkbox, Color, Image, Justify,
    Length, Radio, Slider, Text,
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
    }
}
