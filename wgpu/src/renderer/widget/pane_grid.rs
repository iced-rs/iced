use crate::{Primitive, Renderer};
use iced_native::{pane_grid, Element, Layout, MouseCursor, Point};

impl pane_grid::Renderer for Renderer {
    fn draw<Message>(
        &mut self,
        defaults: &Self::Defaults,
        content: &[(pane_grid::Pane, Element<'_, Message, Self>)],
        layout: Layout<'_>,
        cursor_position: Point,
    ) -> Self::Output {
        let mut mouse_cursor = MouseCursor::OutOfBounds;

        (
            Primitive::Group {
                primitives: content
                    .iter()
                    .zip(layout.children())
                    .map(|((_, pane), layout)| {
                        let (primitive, new_mouse_cursor) =
                            pane.draw(self, defaults, layout, cursor_position);

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
