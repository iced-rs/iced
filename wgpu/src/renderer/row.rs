use crate::{Primitive, Renderer};
use iced_native::{row, Layout, Point, Row};

impl row::Renderer for Renderer {
    fn draw<Message>(
        &mut self,
        row: &Row<'_, Message, Self>,
        layout: Layout<'_>,
        cursor_position: Point,
    ) -> Self::Primitive {
        Primitive::Group {
            primitives: row
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
