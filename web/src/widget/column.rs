use crate::{Align, Element, Widget};

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

impl<'a, Message> Widget<Message> for Column<'a, Message> {}

impl<'a, Message> From<Column<'a, Message>> for Element<'a, Message>
where
    Message: 'static,
{
    fn from(column: Column<'a, Message>) -> Element<'a, Message> {
        Element::new(column)
    }
}
