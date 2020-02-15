use crate::{Primitive, Renderer};
use iced_native::{grid, Element, Layout, MouseCursor, Point};

impl grid::Renderer for Renderer {
    fn draw<Message>(
        &mut self,
        defaults: &Self::Defaults,
        layout: Layout<'_>,
        cursor_position: Point,
        elements: &[Element<'_, Message, Self>],
    ) -> Self::Output {
        let mut mouse_cursor = MouseCursor::OutOfBounds;

        (
            Primitive::Group {
                primitives: {
                    elements
                        .iter()
                        .zip(layout.children())
                        .map(|(element, layout)| {
                            let (primitive, new_mouse_cursor) = element.draw(
                                self,
                                defaults,
                                layout,
                                cursor_position,
                            );

                            if new_mouse_cursor > mouse_cursor {
                                mouse_cursor = new_mouse_cursor;
                            }

                            primitive
                        })
                        .collect()
                },
            },
            mouse_cursor,
        )
    }
}
