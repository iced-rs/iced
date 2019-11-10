use std::hash::Hash;

use crate::{layout, Element, Event, Hasher, Layout, Point, Rectangle, Widget};

/// A container that distributes its contents vertically.
pub type Column<'a, Message, Renderer> =
    iced_core::Column<Element<'a, Message, Renderer>>;

impl<'a, Message, Renderer> Widget<Message, Renderer>
    for Column<'a, Message, Renderer>
where
    Renderer: self::Renderer,
{
    fn layout(&self, renderer: &Renderer, limits: &layout::Limits) -> Layout {
        // TODO
        Layout::new(Rectangle {
            x: 0.0,
            y: 0.0,
            width: 0.0,
            height: 0.0,
        })
    }

    fn on_event(
        &mut self,
        event: Event,
        layout: &Layout,
        cursor_position: Point,
        messages: &mut Vec<Message>,
        renderer: &Renderer,
    ) {
        self.children.iter_mut().zip(layout.children()).for_each(
            |(child, layout)| {
                child.widget.on_event(
                    event,
                    layout,
                    cursor_position,
                    messages,
                    renderer,
                )
            },
        );
    }

    fn draw(
        &self,
        renderer: &mut Renderer,
        layout: &Layout,
        cursor_position: Point,
    ) -> Renderer::Output {
        renderer.draw(&self, layout, cursor_position)
    }

    fn hash_layout(&self, state: &mut Hasher) {
        0.hash(state);
        self.width.hash(state);
        self.height.hash(state);
        self.max_width.hash(state);
        self.max_height.hash(state);
        self.align_self.hash(state);
        self.align_items.hash(state);
        self.justify_content.hash(state);
        self.spacing.hash(state);

        for child in &self.children {
            child.widget.hash_layout(state);
        }
    }
}

pub trait Renderer: crate::Renderer + Sized {
    fn draw<Message>(
        &mut self,
        row: &Column<'_, Message, Self>,
        layout: &Layout,
        cursor_position: Point,
    ) -> Self::Output;
}

impl<'a, Message, Renderer> From<Column<'a, Message, Renderer>>
    for Element<'a, Message, Renderer>
where
    Renderer: 'a + self::Renderer,
    Message: 'static,
{
    fn from(
        column: Column<'a, Message, Renderer>,
    ) -> Element<'a, Message, Renderer> {
        Element::new(column)
    }
}
