use crate::{Primitive, Renderer};
use iced_native::{column, Column, Layout, Point};

impl column::Renderer for Renderer {
    fn draw<Message>(
        &mut self,
        column: &Column<'_, Message, Self>,
        layout: Layout<'_>,
        cursor_position: Point,
    ) -> Self::Primitive {
        Primitive::Group {
            primitives: column
                .children
                .iter()
                .zip(layout.children())
                .map(|(child, layout)| {
                    child.draw(self, layout, cursor_position)
                })
                .collect(),
        }
    }
}
