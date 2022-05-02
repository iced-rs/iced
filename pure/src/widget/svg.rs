//! Display vector graphics in your application.
use crate::widget::{Tree, Widget};
use crate::Element;

use iced_native::layout::{self, Layout};
use iced_native::renderer;
use iced_native::widget::svg;
use iced_native::{Length, Point, Rectangle};

pub use iced_native::svg::Handle;
pub use svg::Svg;

impl<Message, Renderer> Widget<Message, Renderer> for Svg
where
    Renderer: iced_native::svg::Renderer,
{
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
}

impl<'a, Message, Renderer> Into<Element<'a, Message, Renderer>> for Svg
where
    Message: Clone + 'a,
    Renderer: iced_native::svg::Renderer + 'a,
{
    fn into(self) -> Element<'a, Message, Renderer> {
        Element::new(self)
    }
}
