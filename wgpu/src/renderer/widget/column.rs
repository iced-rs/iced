use crate::{Primitive, Renderer};
use iced_native::{column, Column, Layout, MouseCursor, Point};

impl column::Renderer for Renderer {
    fn draw<Message>(
        &mut self,
        column: &Column<'_, Message, Self>,
        layout: &Layout,
        cursor_position: Point,
    ) -> Self::Output {
        let mut mouse_cursor = MouseCursor::OutOfBounds;

        (
            Primitive::Group {
                primitives: column
                    .children
                    .iter()
                    .zip(layout.children())
                    .map(|(child, layout)| {
                        let (primitive, new_mouse_cursor) =
                            child.draw(self, layout, cursor_position);

                        if new_mouse_cursor > mouse_cursor {
                            mouse_cursor = new_mouse_cursor;
                        }

                        primitive
                    })
                    .collect(),
            },
            mouse_cursor,
        )
    }
}
