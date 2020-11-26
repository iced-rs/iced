use crate::{Bus, Color, Css, Widget};

use dodrio::bumpalo;
use std::rc::Rc;

/// A generic [`Widget`].
///
/// It is useful to build composable user interfaces that do not leak
/// implementation details in their __view logic__.
///
/// If you have a [built-in widget], you should be able to use `Into<Element>`
/// to turn it into an [`Element`].
///
/// [built-in widget]: mod@crate::widget
#[allow(missing_debug_implementations)]
pub struct Element<'a, Message> {
    pub(crate) widget: Box<dyn Widget<Message> + 'a>,
}

impl<'a, Message> Element<'a, Message> {
    /// Create a new [`Element`] containing the given [`Widget`].
    pub fn new(widget: impl Widget<Message> + 'a) -> Self {
        Self {
            widget: Box::new(widget),
        }
    }

    /// Applies a transformation to the produced message of the [`Element`].
    ///
    /// This method is useful when you want to decouple different parts of your
    /// UI and make them __composable__.
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

    /// Marks the [`Element`] as _to-be-explained_.
    pub fn explain(self, _color: Color) -> Element<'a, Message> {
        self
    }

    /// Produces a VDOM node for the [`Element`].
    pub fn node<'b>(
        &self,
        bump: &'b bumpalo::Bump,
        bus: &Bus<Message>,
        style_sheet: &mut Css<'b>,
    ) -> dodrio::Node<'b> {
        self.widget.node(bump, bus, style_sheet)
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
        style_sheet: &mut Css<'b>,
    ) -> dodrio::Node<'b> {
        self.widget
            .node(bump, &bus.map(self.mapper.clone()), style_sheet)
    }
}
