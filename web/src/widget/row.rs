use crate::{Element, Widget};

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

impl<'a, Message> Widget<Message> for Row<'a, Message> {}

impl<'a, Message> From<Row<'a, Message>> for Element<'a, Message>
where
    Message: 'static,
{
    fn from(column: Row<'a, Message>) -> Element<'a, Message> {
        Element::new(column)
    }
}
