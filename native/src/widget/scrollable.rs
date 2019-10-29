use crate::{
    column, input::mouse, Element, Event, Hasher, Layout, Node, Point,
    Rectangle, Style, Widget,
};

pub use iced_core::scrollable::State;

use std::hash::Hash;

/// A scrollable [`Column`].
///
/// [`Column`]: ../column/struct.Column.html
pub type Scrollable<'a, Message, Renderer> =
    iced_core::Scrollable<'a, Element<'a, Message, Renderer>>;

impl<'a, Message, Renderer> Widget<Message, Renderer>
    for Scrollable<'a, Message, Renderer>
where
    Renderer: self::Renderer + column::Renderer,
{
    fn node(&self, renderer: &Renderer) -> Node {
        let mut content = self.content.node(renderer);

        {
            let mut style = content.0.style();
            style.flex_shrink = 0.0;

            content.0.set_style(style);
        }

        let mut style = Style::default()
            .width(self.content.width)
            .max_width(self.content.max_width)
            .height(self.height)
            .max_height(self.max_height)
            .align_self(self.align_self);

        style.0.flex_direction = stretch::style::FlexDirection::Column;

        Node::with_children(style, vec![content])
    }

    fn on_event(
        &mut self,
        event: Event,
        layout: Layout<'_>,
        cursor_position: Point,
        messages: &mut Vec<Message>,
    ) {
        let bounds = layout.bounds();
        let is_mouse_over = bounds.contains(cursor_position);

        let content = layout.children().next().unwrap();
        let content_bounds = content.bounds();

        if is_mouse_over {
            match event {
                Event::Mouse(mouse::Event::WheelScrolled {
                    delta_y, ..
                }) => {
                    self.state.scroll(delta_y, bounds, content_bounds);
                }
                _ => {}
            }
        }

        let cursor_position = if is_mouse_over {
            Point::new(
                cursor_position.x,
                cursor_position.y
                    + self.state.offset(bounds, content_bounds) as f32,
            )
        } else {
            // TODO: Make `cursor_position` an `Option<Point>` so we can encode
            // cursor unavailability.
            // This will probably happen naturally once we add multi-window
            // support.
            Point::new(cursor_position.x, -1.0)
        };

        self.content
            .on_event(event, content, cursor_position, messages)
    }

    fn draw(
        &self,
        renderer: &mut Renderer,
        layout: Layout<'_>,
        cursor_position: Point,
    ) -> Renderer::Output {
        let bounds = layout.bounds();
        let content_layout = layout.children().next().unwrap();

        self::Renderer::draw(
            renderer,
            &self,
            bounds,
            content_layout,
            cursor_position,
        )
    }

    fn hash_layout(&self, state: &mut Hasher) {
        std::any::TypeId::of::<Scrollable<'static, (), ()>>().hash(state);

        self.height.hash(state);
        self.max_height.hash(state);
        self.align_self.hash(state);

        self.content.hash_layout(state)
    }
}

pub trait Renderer: crate::Renderer + Sized {
    fn draw<Message>(
        &mut self,
        scrollable: &Scrollable<'_, Message, Self>,
        bounds: Rectangle,
        content_layout: Layout<'_>,
        cursor_position: Point,
    ) -> Self::Output;
}

impl<'a, Message, Renderer> From<Scrollable<'a, Message, Renderer>>
    for Element<'a, Message, Renderer>
where
    Renderer: 'a + self::Renderer + column::Renderer,
    Message: 'static,
{
    fn from(
        scrollable: Scrollable<'a, Message, Renderer>,
    ) -> Element<'a, Message, Renderer> {
        Element::new(scrollable)
    }
}
