use crate::{Element, Widget};

pub use iced::button::{Class, State};

pub type Button<'a, Message> = iced::Button<'a, Message>;

impl<'a, Message> Widget<Message> for Button<'a, Message> {}

impl<'a, Message> From<Button<'a, Message>> for Element<'a, Message>
where
    Message: 'static,
{
    fn from(button: Button<'a, Message>) -> Element<'a, Message> {
        Element::new(button)
    }
}
