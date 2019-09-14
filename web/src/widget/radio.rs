use crate::{Color, Element, Widget};

pub type Radio<Message> = iced::Radio<Color, Message>;

impl<Message> Widget<Message> for Radio<Message> {}

impl<'a, Message> From<Radio<Message>> for Element<'a, Message>
where
    Message: 'static,
{
    fn from(radio: Radio<Message>) -> Element<'a, Message> {
        Element::new(radio)
    }
}
