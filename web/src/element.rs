use crate::{Bus, Color, Widget};

use dodrio::bumpalo;
use std::rc::Rc;

pub struct Element<'a, Message> {
    pub(crate) widget: Box<dyn Widget<Message> + 'a>,
}

impl<'a, Message> Element<'a, Message> {
    pub fn new(widget: impl Widget<Message> + 'a) -> Self {
        Self {
            widget: Box::new(widget),
        }
    }

    pub fn node<'b>(
        &self,
        bump: &'b bumpalo::Bump,
        bus: &Bus<Message>,
    ) -> dodrio::Node<'b> {
        self.widget.node(bump, bus)
    }

    pub fn explain(self, _color: Color) -> Element<'a, Message> {
        self
    }

    pub fn map<F, B>(self, f: F) -> Element<'a, B>
    where
        Message: 'static,
        B: 'static,
        F: 'static + Fn(Message) -> B,
    {
        Element {
            widget: Box::new(Map::new(self.widget, f)),
        }
    }
}

struct Map<'a, A, B> {
    widget: Box<dyn Widget<A> + 'a>,
    mapper: Rc<Box<dyn Fn(A) -> B>>,
}

impl<'a, A, B> Map<'a, A, B> {
    pub fn new<F>(widget: Box<dyn Widget<A> + 'a>, mapper: F) -> Map<'a, A, B>
    where
        F: 'static + Fn(A) -> B,
    {
        Map {
            widget,
            mapper: Rc::new(Box::new(mapper)),
        }
    }
}

impl<'a, A, B> Widget<B> for Map<'a, A, B>
where
    A: 'static,
    B: 'static,
{
    fn node<'b>(
        &self,
        bump: &'b bumpalo::Bump,
        bus: &Bus<B>,
    ) -> dodrio::Node<'b> {
        self.widget.node(bump, &bus.map(self.mapper.clone()))
    }
}
