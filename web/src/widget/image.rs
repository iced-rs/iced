use crate::{Bus, Element, Widget};

use dodrio::bumpalo;

pub type Image<'a> = iced::Image<&'a str>;

impl<'a, Message> Widget<Message> for Image<'a> {
    fn node<'b>(
        &self,
        bump: &'b bumpalo::Bump,
        _bus: &Bus<Message>,
    ) -> dodrio::Node<'b> {
        use dodrio::builder::*;

        let src = bumpalo::format!(in bump, "{}", self.image);

        let mut image = img(bump).attr("src", src.into_bump_str());

        if let Some(width) = self.width {
            let width = bumpalo::format!(in bump, "{}", width);
            image = image.attr("width", width.into_bump_str());
        }

        image.finish()
    }
}

impl<'a, Message> From<Image<'a>> for Element<'a, Message> {
    fn from(image: Image<'a>) -> Element<'a, Message> {
        Element::new(image)
    }
}
