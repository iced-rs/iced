use crate::{Primitive, Renderer};
use iced_native::{table, Element, Layout, MouseCursor, Point};

impl table::Renderer for Renderer {
    fn draw<Message>(
        &mut self,
        defaults: &Self::Defaults,
        layout: Layout<'_>,
        cursor_position: Point,
        rows: &[Vec<Element<'_, Message, Self>>],
    ) -> Self::Output {
        let mut mouse_cursor = MouseCursor::OutOfBounds;
        let mut primitives = Vec::new();

        for (row, layout) in rows.iter().zip(layout.children()) {
            for (cell, layout) in row.iter().zip(layout.children()) {
                let (primitive, new_mouse_cursor) =
                    cell.draw(self, defaults, layout, cursor_position);

                if new_mouse_cursor > mouse_cursor {
                    mouse_cursor = new_mouse_cursor;
                }

                primitives.push(primitive);
            }
        }

        (Primitive::Group { primitives }, mouse_cursor)
    }
}
