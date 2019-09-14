use crate::{Element, Widget};

pub use iced::slider::State;

pub type Slider<'a, Message> = iced::Slider<'a, Message>;

impl<'a, Message> Widget<Message> for Slider<'a, Message> {}

impl<'a, Message> From<Slider<'a, Message>> for Element<'a, Message>
where
    Message: 'static,
{
    fn from(slider: Slider<'a, Message>) -> Element<'a, Message> {
        Element::new(slider)
    }
}
