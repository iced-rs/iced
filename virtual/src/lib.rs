mod element;
pub mod widget;

pub use element::Element;
pub use widget::Widget;

use iced_native::layout::{self, Layout};
use iced_native::renderer;
use iced_native::{Hasher, Length, Point, Rectangle};

pub struct Virtual<'a, Message, Renderer> {
    state: &'a mut State<Message, Renderer>,
}

pub struct State<Message, Renderer> {
    widget_tree: widget::Tree<Message, Renderer>,
    last_element: Element<Message, Renderer>,
}

impl<'a, Message, Renderer> iced_native::Widget<Message, Renderer>
    for Virtual<'a, Message, Renderer>
where
    Renderer: iced_native::Renderer,
{
    fn width(&self) -> Length {
        self.state.widget_tree.width()
    }

    fn height(&self) -> Length {
        self.state.widget_tree.height()
    }

    fn layout(
        &self,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        self.state.widget_tree.layout(renderer, limits)
    }

    fn hash_layout(&self, state: &mut Hasher) {
        self.state.widget_tree.hash_layout(state)
    }

    fn draw(
        &self,
        renderer: &mut Renderer,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor_position: Point,
        viewport: &Rectangle,
    ) {
        self.state.widget_tree.draw(
            renderer,
            style,
            layout,
            cursor_position,
            viewport,
        )
    }
}
