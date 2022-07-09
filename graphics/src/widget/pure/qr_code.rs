//! Encode and display information in a QR code.
pub use crate::qr_code::*;

use crate::{Backend, Renderer};

use iced_native::layout::{self, Layout};
use iced_native::renderer;
use iced_native::{Length, Point, Rectangle};
use iced_pure::widget::tree::Tree;
use iced_pure::{Element, Widget};

impl<'a, Message, B, T> Widget<Message, Renderer<B, T>> for QRCode<'a>
where
    B: Backend,
{
    fn width(&self) -> Length {
        <Self as iced_native::Widget<Message, Renderer<B, T>>>::width(self)
    }

    fn height(&self) -> Length {
        <Self as iced_native::Widget<Message, Renderer<B, T>>>::height(self)
    }

    fn layout(
        &self,
        renderer: &Renderer<B, T>,
        limits: &layout::Limits,
    ) -> layout::Node {
        <Self as iced_native::Widget<Message, Renderer<B, T>>>::layout(
            self, renderer, limits,
        )
    }

    fn draw(
        &self,
        _tree: &Tree,
        renderer: &mut Renderer<B, T>,
        theme: &T,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor_position: Point,
        viewport: &Rectangle,
    ) {
        <Self as iced_native::Widget<Message, Renderer<B, T>>>::draw(
            self,
            renderer,
            theme,
            style,
            layout,
            cursor_position,
            viewport,
        )
    }
}

impl<'a, Message, B, T> From<QRCode<'a>>
    for Element<'a, Message, Renderer<B, T>>
where
    B: Backend,
{
    fn from(qr_code: QRCode<'a>) -> Self {
        Self::new(qr_code)
    }
}
