use crate::{Bus, Element, Length, Widget};

use dodrio::bumpalo;

pub use iced_core::Image;

impl<Message> Widget<Message> for Image {
    fn node<'b>(
        &self,
        bump: &'b bumpalo::Bump,
        _bus: &Bus<Message>,
    ) -> dodrio::Node<'b> {
        use dodrio::builder::*;

        let src = bumpalo::format!(in bump, "{}", self.path);

        let mut image = img(bump).attr("src", src.into_bump_str());

        match self.width {
            Length::Shrink => {}
            Length::Fill => {
                image = image.attr("width", "100%");
            }
            Length::Units(px) => {
                image = image.attr(
                    "width",
                    bumpalo::format!(in bump, "{}px", px).into_bump_str(),
                );
            }
        }

        // TODO: Complete styling

        image.finish()
    }
}

impl<'a, Message> From<Image> for Element<'a, Message> {
    fn from(image: Image) -> Element<'a, Message> {
        Element::new(image)
    }
}
