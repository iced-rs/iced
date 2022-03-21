//! Encode and display information in a QR code.
pub use crate::qr_code::*;

use crate::{Backend, Renderer};

use iced_native::layout::{self, Layout};
use iced_native::renderer;
use iced_native::{Length, Point, Rectangle};
use iced_pure::widget::tree::Tree;
use iced_pure::{Element, Widget};

impl<'a, Message, B> Widget<Message, Renderer<B>> for QRCode<'a>
where
    B: Backend,
{
    fn width(&self) -> Length {
        <Self as iced_native::Widget<Message, Renderer<B>>>::width(self)
    }

    fn height(&self) -> Length {
        <Self as iced_native::Widget<Message, Renderer<B>>>::height(self)
    }

    fn layout(
        &self,
        renderer: &Renderer<B>,
        limits: &layout::Limits,
    ) -> layout::Node {
        <Self as iced_native::Widget<Message, Renderer<B>>>::layout(
            self, renderer, limits,
        )
    }

    fn draw(
        &self,
        _tree: &Tree,
        renderer: &mut Renderer<B>,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor_position: Point,
        viewport: &Rectangle,
    ) {
        <Self as iced_native::Widget<Message, Renderer<B>>>::draw(
            self,
            renderer,
            style,
            layout,
            cursor_position,
            viewport,
        )
    }
}

impl<'a, Message, B> Into<Element<'a, Message, Renderer<B>>> for QRCode<'a>
where
    B: Backend,
{
    fn into(self) -> Element<'a, Message, Renderer<B>> {
        Element::new(self)
    }
}
