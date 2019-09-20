use std::hash::Hash;

use crate::{
    Element, Event, Hasher, Layout, MouseCursor, Node, Point, Style, Widget,
};

/// A container that distributes its contents horizontally.
pub type Row<'a, Message, Renderer> =
    iced_core::Row<Element<'a, Message, Renderer>>;

impl<'a, Message, Renderer> Widget<Message, Renderer>
    for Row<'a, Message, Renderer>
{
    fn node(&self, renderer: &mut Renderer) -> Node {
        let mut children: Vec<Node> = self
            .children
            .iter()
            .map(|child| {
                let mut node = child.widget.node(renderer);

                let mut style = node.0.style();
                style.margin.end =
                    stretch::style::Dimension::Points(f32::from(self.spacing));

                node.0.set_style(style);
                node
            })
            .collect();

        if let Some(node) = children.last_mut() {
            let mut style = node.0.style();
            style.margin.end = stretch::style::Dimension::Undefined;

            node.0.set_style(style);
        }

        let mut style = Style::default()
            .width(self.width)
            .height(self.height)
            .max_width(self.max_width)
            .max_height(self.max_height)
            .padding(self.padding)
            .align_self(self.align_self)
            .align_items(self.align_items)
            .justify_content(self.justify_content);

        style.0.flex_direction = stretch::style::FlexDirection::Row;

        Node::with_children(style, children)
    }

    fn on_event(
        &mut self,
        event: Event,
        layout: Layout<'_>,
        cursor_position: Point,
        messages: &mut Vec<Message>,
    ) {
        self.children.iter_mut().zip(layout.children()).for_each(
            |(child, layout)| {
                child
                    .widget
                    .on_event(event, layout, cursor_position, messages)
            },
        );
    }

    fn draw(
        &self,
        renderer: &mut Renderer,
        layout: Layout<'_>,
        cursor_position: Point,
    ) -> MouseCursor {
        let mut cursor = MouseCursor::OutOfBounds;

        self.children.iter().zip(layout.children()).for_each(
            |(child, layout)| {
                let new_cursor =
                    child.widget.draw(renderer, layout, cursor_position);

                if new_cursor != MouseCursor::OutOfBounds {
                    cursor = new_cursor;
                }
            },
        );

        cursor
    }

    fn hash_layout(&self, state: &mut Hasher) {
        1.hash(state);
        self.width.hash(state);
        self.height.hash(state);
        self.max_width.hash(state);
        self.max_height.hash(state);
        self.align_self.hash(state);
        self.align_items.hash(state);
        self.justify_content.hash(state);
        self.spacing.hash(state);
        self.spacing.hash(state);

        for child in &self.children {
            child.widget.hash_layout(state);
        }
    }
}

impl<'a, Message, Renderer> From<Row<'a, Message, Renderer>>
    for Element<'a, Message, Renderer>
where
    Renderer: 'a,
    Message: 'static,
{
    fn from(row: Row<'a, Message, Renderer>) -> Element<'a, Message, Renderer> {
        Element::new(row)
    }
}
