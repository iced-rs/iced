use crate::widget::{Tree, Widget};

use iced_native::layout::{self, Layout};
use iced_native::renderer;
use iced_native::widget::image;
use iced_native::{Hasher, Length, Point, Rectangle};

use std::any::{self, Any};
use std::hash::Hash;

pub use image::Image;

impl<Message, Renderer, Handle> Widget<Message, Renderer> for Image<Handle>
where
    Handle: Clone + Hash,
    Renderer: iced_native::image::Renderer<Handle = Handle>,
{
    fn tag(&self) -> any::TypeId {
        any::TypeId::of::<()>()
    }

    fn state(&self) -> Box<dyn Any> {
        Box::new(())
    }

    fn children_state(&self) -> Vec<Tree> {
        Vec::new()
    }

    fn diff(&self, _tree: &mut Tree) {}

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
