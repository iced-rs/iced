use crate::{Primitive, Renderer};
use iced_native::{panes, Element, Layout, MouseCursor, Point};

impl panes::Renderer for Renderer {
    fn draw<Message>(
        &mut self,
        defaults: &Self::Defaults,
        content: &[Element<'_, Message, Self>],
        layout: Layout<'_>,
        cursor_position: Point,
    ) -> Self::Output {
        let mut mouse_cursor = MouseCursor::OutOfBounds;

        (
            Primitive::Group {
                primitives: content
                    .iter()
                    .zip(layout.children())
                    .map(|(child, layout)| {
                        let (primitive, new_mouse_cursor) =
                            child.draw(self, defaults, layout, cursor_position);

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
