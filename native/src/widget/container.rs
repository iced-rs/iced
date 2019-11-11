use std::hash::Hash;

use crate::{
    layout, Element, Event, Hasher, Layout, Length, Point, Size, Widget,
};

/// A container that distributes its contents vertically.
pub type Container<'a, Message, Renderer> =
    iced_core::Container<Element<'a, Message, Renderer>>;

impl<'a, Message, Renderer> Widget<Message, Renderer>
    for Container<'a, Message, Renderer>
where
    Renderer: crate::Renderer,
{
    fn width(&self) -> Length {
        self.width
    }

    fn layout(
        &self,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let limits = limits
            .loose()
            .max_width(self.max_width)
            .max_height(self.max_height)
            .width(self.width)
            .height(self.height);

        let mut content = self.content.layout(renderer, &limits);
        let size = limits.resolve(content.size());

        content.align(self.horizontal_alignment, self.vertical_alignment, size);

        layout::Node::with_children(size, vec![content])
    }

    fn on_event(
        &mut self,
        event: Event,
        layout: Layout<'_>,
        cursor_position: Point,
        messages: &mut Vec<Message>,
        renderer: &Renderer,
    ) {
        self.content.widget.on_event(
            event,
            layout.children().next().unwrap(),
            cursor_position,
            messages,
            renderer,
        )
    }

    fn draw(
        &self,
        renderer: &mut Renderer,
        layout: Layout<'_>,
        cursor_position: Point,
    ) -> Renderer::Output {
        self.content.draw(
            renderer,
            layout.children().next().unwrap(),
            cursor_position,
        )
    }

    fn hash_layout(&self, state: &mut Hasher) {
        0.hash(state);
        self.width.hash(state);
        self.height.hash(state);
        self.max_width.hash(state);
        self.max_height.hash(state);
        self.padding.hash(state);

        self.content.hash_layout(state);
    }
}

impl<'a, Message, Renderer> From<Container<'a, Message, Renderer>>
    for Element<'a, Message, Renderer>
where
    Renderer: 'a + crate::Renderer,
    Message: 'static,
{
    fn from(
        column: Container<'a, Message, Renderer>,
    ) -> Element<'a, Message, Renderer> {
        Element::new(column)
    }
}
