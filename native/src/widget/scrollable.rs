use crate::{
    column, input::mouse, Element, Event, Hasher, Layout, Node, Point, Style,
    Widget,
};

pub use iced_core::scrollable::State;

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
            .align_self(self.align_self)
            .align_items(self.align_items);

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
                    // TODO: Configurable speed (?)
                    self.state.offset = (self.state.offset as i32
                        - delta_y.round() as i32 * 15)
                        .max(0)
                        .min((content_bounds.height - bounds.height) as i32)
                        as u32;
                }
                _ => {}
            }
        }

        let cursor_position = if is_mouse_over {
            Point::new(
                cursor_position.x,
                cursor_position.y + self.state.offset as f32,
            )
        } else {
            Point::new(cursor_position.x, -1.0)
        };

        self.content.on_event(
            event,
            layout.children().next().unwrap(),
            cursor_position,
            messages,
        )
    }

    fn draw(
        &self,
        renderer: &mut Renderer,
        layout: Layout<'_>,
        cursor_position: Point,
    ) -> Renderer::Output {
        self::Renderer::draw(renderer, &self, layout, cursor_position)
    }

    fn hash_layout(&self, state: &mut Hasher) {
        self.content.hash_layout(state)
    }
}

pub trait Renderer: crate::Renderer + Sized {
    fn draw<Message>(
        &mut self,
        scrollable: &Scrollable<'_, Message, Self>,
        layout: Layout<'_>,
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
