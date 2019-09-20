use crate::{Bus, Element, Widget};

use dodrio::bumpalo;

pub type Column<'a, Message> = iced_core::Column<Element<'a, Message>>;

impl<'a, Message> Widget<Message> for Column<'a, Message> {
    fn node<'b>(
        &self,
        bump: &'b bumpalo::Bump,
        publish: &Bus<Message>,
    ) -> dodrio::Node<'b> {
        use dodrio::builder::*;

        let children: Vec<_> = self
            .children
            .iter()
            .map(|element| element.widget.node(bump, publish))
            .collect();

        div(bump)
            .attr("style", "display: flex; flex-direction: column")
            .children(children)
            .finish()
    }
}

impl<'a, Message> From<Column<'a, Message>> for Element<'a, Message>
where
    Message: 'static,
{
    fn from(column: Column<'a, Message>) -> Element<'a, Message> {
        Element::new(column)
    }
}
