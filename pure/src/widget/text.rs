use crate::{Element, Tree, Widget};

use iced_native::layout::{self, Layout};
use iced_native::renderer;
use iced_native::text;
use iced_native::{Hasher, Length, Point, Rectangle};

use std::any::{self, Any};

pub use iced_native::widget::Text;

impl<Message, Renderer> Widget<Message, Renderer> for Text<Renderer>
where
    Renderer: text::Renderer,
{
    fn tag(&self) -> any::TypeId {
        any::TypeId::of::<()>()
    }

    fn state(&self) -> Box<dyn Any> {
        Box::new(())
    }

    fn children(&self) -> &[Element<Message, Renderer>] {
        &[]
    }

    fn width(&self) -> Length {
        <Self as iced_native::Widget<Message, Renderer>>::width(self)
    }

    fn height(&self) -> Length {
        <Self as iced_native::Widget<Message, Renderer>>::height(self)
    }

    fn layout(
        &self,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        <Self as iced_native::Widget<Message, Renderer>>::layout(
            self, renderer, limits,
        )
    }

    fn draw(
        &self,
        _tree: &Tree,
        renderer: &mut Renderer,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor_position: Point,
        viewport: &Rectangle,
    ) {
        <Self as iced_native::Widget<Message, Renderer>>::draw(
            self,
            renderer,
            style,
            layout,
            cursor_position,
            viewport,
        )
    }

    fn hash_layout(&self, state: &mut Hasher) {
        <Self as iced_native::Widget<Message, Renderer>>::hash_layout(
            self, state,
        )
    }
}

impl<'a, Message, Renderer> Into<Element<'a, Message, Renderer>>
    for Text<Renderer>
where
    Renderer: text::Renderer + 'static,
{
    fn into(self) -> Element<'a, Message, Renderer> {
        Element::new(self)
    }
}

impl<'a, Message, Renderer> Into<Element<'a, Message, Renderer>>
    for &'static str
where
    Renderer: text::Renderer + 'static,
{
    fn into(self) -> Element<'a, Message, Renderer> {
        Text::new(self).into()
    }
}
