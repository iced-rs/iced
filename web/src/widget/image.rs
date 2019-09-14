use crate::{Element, Widget};

pub type Image<'a> = iced::Image<&'a str>;

impl<'a, Message> Widget<Message> for Image<'a> {}

impl<'a, Message> From<Image<'a>> for Element<'a, Message> {
    fn from(image: Image<'a>) -> Element<'a, Message> {
        Element::new(image)
    }
}
