use crate::{Align, Bus, Element, Widget};

use dodrio::bumpalo;

pub struct Column<'a, Message> {
    children: Vec<Element<'a, Message>>,
}

impl<'a, Message> Column<'a, Message> {
    pub fn new() -> Self {
        Self {
            children: Vec::new(),
        }
    }

    pub fn spacing(self, _spacing: u16) -> Self {
        self
    }

    pub fn padding(self, _padding: u16) -> Self {
        self
    }

    pub fn max_width(self, _max_width: u16) -> Self {
        self
    }

    pub fn align_items(self, _align: Align) -> Self {
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

        div(bump).children(children).finish()
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
