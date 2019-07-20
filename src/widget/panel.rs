use std::hash::Hash;

use crate::graphics::{Point, Rectangle};
use crate::ui::core::{
    Event, Hasher, Layout, MouseCursor, Node, Style, Widget,
};

pub struct Panel<'a, Message, Renderer> {
    style: Style,
    content: Box<Widget<Message, Renderer> + 'a>,
}

impl<'a, Message, Renderer> Panel<'a, Message, Renderer> {
    pub fn new(content: impl Widget<Message, Renderer> + 'a) -> Self {
        Panel {
            style: Style::default().padding(20),
            content: Box::new(content),
        }
    }

    pub fn width(mut self, width: u32) -> Self {
        self.style = self.style.width(width);
        self
    }

    pub fn max_width(mut self, max_width: u32) -> Self {
        self.style = self.style.max_width(max_width);
        self
    }
}

impl<'a, Message, Renderer> Widget<Message, Renderer>
    for Panel<'a, Message, Renderer>
where
    Renderer: self::Renderer,
{
    fn node(&self, renderer: &Renderer) -> Node {
        Node::with_children(self.style, vec![self.content.node(renderer)])
    }

    fn on_event(
        &mut self,
        event: Event,
        layout: Layout,
        cursor_position: Point,
        messages: &mut Vec<Message>,
    ) {
        [&mut self.content]
            .iter_mut()
            .zip(layout.children())
            .for_each(|(child, layout)| {
                child.on_event(event, layout, cursor_position, messages)
            });
    }

    fn draw(
        &self,
        renderer: &mut Renderer,
        layout: Layout,
        cursor_position: Point,
    ) -> MouseCursor {
        let bounds = layout.bounds();
        let mut cursor = MouseCursor::OutOfBounds;
        renderer.draw(bounds);

        [&self.content].iter().zip(layout.children()).for_each(
            |(child, layout)| {
                let new_cursor = child.draw(renderer, layout, cursor_position);

                if new_cursor != MouseCursor::OutOfBounds {
                    cursor = new_cursor;
                }
            },
        );

        if cursor == MouseCursor::OutOfBounds {
            if bounds.contains(cursor_position) {
                MouseCursor::Idle
            } else {
                MouseCursor::OutOfBounds
            }
        } else {
            cursor
        }
    }

    fn hash(&self, state: &mut Hasher) {
        self.style.hash(state);
    }
}

pub trait Renderer {
    fn draw(&mut self, bounds: Rectangle<f32>);
}
