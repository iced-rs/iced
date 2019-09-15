use crate::{Bus, Element, Widget};

use dodrio::bumpalo;

pub struct Row<'a, Message> {
    children: Vec<Element<'a, Message>>,
}

impl<'a, Message> Row<'a, Message> {
    pub fn new() -> Self {
        Self {
            children: Vec::new(),
        }
    }

    pub fn spacing(self, _spacing: u16) -> Self {
        self
    }

    pub fn push<E>(mut self, element: E) -> Self
    where
        E: Into<Element<'a, Message>>,
    {
        self.children.push(element.into());
        self
    }
}

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

        div(bump).children(children).finish()
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
