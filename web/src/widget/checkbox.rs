use crate::{Color, Element, Widget};

pub type Checkbox<Message> = iced::Checkbox<Color, Message>;

impl<Message> Widget<Message> for Checkbox<Message> {}

impl<'a, Message> From<Checkbox<Message>> for Element<'a, Message>
where
    Message: 'static,
{
    fn from(checkbox: Checkbox<Message>) -> Element<'a, Message> {
        Element::new(checkbox)
    }
}
