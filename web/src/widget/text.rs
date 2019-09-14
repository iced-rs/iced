use crate::{Color, Element, Widget};

pub use iced::text::HorizontalAlignment;

pub type Text = iced::Text<Color>;

impl<'a, Message> Widget<Message> for Text {}

impl<'a, Message> From<Text> for Element<'a, Message> {
    fn from(text: Text) -> Element<'a, Message> {
        Element::new(text)
    }
}
