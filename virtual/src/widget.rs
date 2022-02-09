mod button;

pub use button::Button;

use iced_native::layout::{self, Layout};
use iced_native::renderer;
use iced_native::{Hasher, Length, Point, Rectangle};

pub trait Widget<Message, Renderer> {}

pub(crate) enum Tree<Message, Renderer> {
    Node {
        widget: Box<dyn Widget<Message, Renderer>>,
        children: Vec<Tree<Message, Renderer>>,
    },
    Leaf {
        widget: Box<dyn Widget<Message, Renderer>>,
    },
}

impl<Message, Renderer> Tree<Message, Renderer> {
    pub fn width(&self) -> Length {
        unimplemented! {}
    }

    pub fn height(&self) -> Length {
        unimplemented! {}
    }

    pub fn hash_layout(&self, state: &mut Hasher) {
        unimplemented! {}
    }

    pub fn layout(
        &self,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        unimplemented! {}
    }

    pub fn draw(
        &self,
        renderer: &mut Renderer,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor_position: Point,
        viewport: &Rectangle,
    ) {
        unimplemented! {}
    }
}
