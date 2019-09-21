use crate::{Bus, Element, Widget};

use dodrio::bumpalo;

pub type Row<'a, Message> = iced_core::Row<Element<'a, Message>>;

impl<'a, Message> Widget<Message> for Row<'a, Message> {
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

        // TODO: Complete styling
        div(bump)
            .attr("style", "display: flex; flex-direction: row")
            .children(children)
            .finish()
    }
}

impl<'a, Message> From<Row<'a, Message>> for Element<'a, Message>
where
    Message: 'static,
{
    fn from(column: Row<'a, Message>) -> Element<'a, Message> {
        Element::new(column)
    }
}
