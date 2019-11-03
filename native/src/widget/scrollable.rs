use crate::{
    column,
    input::{mouse, ButtonState},
    Element, Event, Hasher, Layout, Node, Point, Rectangle, Style, Widget,
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
        renderer: &Renderer,
    ) {
        let bounds = layout.bounds();
        let is_mouse_over = bounds.contains(cursor_position);

        let content = layout.children().next().unwrap();
        let content_bounds = content.bounds();

        let is_mouse_over_scrollbar = renderer.is_mouse_over_scrollbar(
            bounds,
            content_bounds,
            cursor_position,
        );

        // TODO: Event capture. Nested scrollables should capture scroll events.
        if is_mouse_over {
            match event {
                Event::Mouse(mouse::Event::WheelScrolled { delta }) => {
                    match delta {
                        mouse::ScrollDelta::Lines { y, .. } => {
                            // TODO: Configurable speed (?)
                            self.state.scroll(y * 15.0, bounds, content_bounds);
                        }
                        mouse::ScrollDelta::Pixels { y, .. } => {
                            self.state.scroll(y, bounds, content_bounds);
                        }
                    }
                }
                _ => {}
            }
        }

        if self.state.is_scrollbar_grabbed() || is_mouse_over_scrollbar {
            match event {
                Event::Mouse(mouse::Event::Input {
                    button: mouse::Button::Left,
                    state,
                }) => match state {
                    ButtonState::Pressed => {
                        self.state.scroll_to(
                            cursor_position.y / (bounds.y + bounds.height),
                            bounds,
                            content_bounds,
                        );

                        self.state.scrollbar_grabbed_at = Some(cursor_position);
                    }
                    ButtonState::Released => {
                        self.state.scrollbar_grabbed_at = None;
                    }
                },
                Event::Mouse(mouse::Event::CursorMoved { .. }) => {
                    if let Some(scrollbar_grabbed_at) =
                        self.state.scrollbar_grabbed_at
                    {
                        let ratio = content_bounds.height / bounds.height;
                        let delta = scrollbar_grabbed_at.y - cursor_position.y;

                        self.state.scroll(
                            delta * ratio,
                            bounds,
                            content_bounds,
                        );

                        self.state.scrollbar_grabbed_at = Some(cursor_position);
                    }
                }
                _ => {}
            }
        }

        let cursor_position = if is_mouse_over
            && !(is_mouse_over_scrollbar
                || self.state.scrollbar_grabbed_at.is_some())
        {
            Point::new(
                cursor_position.x,
                cursor_position.y
                    + self.state.offset(bounds, content_bounds) as f32,
            )
        } else {
            // TODO: Make `cursor_position` an `Option<Point>` so we can encode
            // cursor availability.
            // This will probably happen naturally once we add multi-window
            // support.
            Point::new(cursor_position.x, -1.0)
        };

        self.content.on_event(
            event,
            content,
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
    fn is_mouse_over_scrollbar(
        &self,
        bounds: Rectangle,
        content_bounds: Rectangle,
        cursor_position: Point,
    ) -> bool;

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
