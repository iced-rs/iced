use crate::{Primitive, Renderer};
use iced_native::{
    pane_grid::{self, Pane},
    Element, Layout, MouseCursor, Point, Rectangle, Vector,
};

impl pane_grid::Renderer for Renderer {
    fn draw<Message>(
        &mut self,
        defaults: &Self::Defaults,
        content: &[(Pane, Element<'_, Message, Self>)],
        dragging: Option<Pane>,
        layout: Layout<'_>,
        cursor_position: Point,
    ) -> Self::Output {
        let mut mouse_cursor = MouseCursor::OutOfBounds;
        let mut dragged_pane = None;

        let mut panes: Vec<_> = content
            .iter()
            .zip(layout.children())
            .enumerate()
            .map(|(i, ((id, pane), layout))| {
                let (primitive, new_mouse_cursor) =
                    pane.draw(self, defaults, layout, cursor_position);

                if new_mouse_cursor > mouse_cursor {
                    mouse_cursor = new_mouse_cursor;
                }

                if Some(*id) == dragging {
                    dragged_pane = Some((i, layout));
                }

                primitive
            })
            .collect();

        let primitives = if let Some((index, layout)) = dragged_pane {
            let pane = panes.remove(index);
            let bounds = layout.bounds();

            // TODO: Fix once proper layering is implemented.
            // This is a pretty hacky way to achieve layering.
            let clip = Primitive::Clip {
                bounds: Rectangle {
                    x: cursor_position.x - bounds.width / 2.0,
                    y: cursor_position.y - bounds.height / 2.0,
                    width: bounds.width + 0.5,
                    height: bounds.height + 0.5,
                },
                offset: Vector::new(0, 0),
                content: Box::new(Primitive::Cached {
                    origin: Point::new(
                        cursor_position.x - bounds.x - bounds.width / 2.0,
                        cursor_position.y - bounds.y - bounds.height / 2.0,
                    ),
                    cache: std::sync::Arc::new(pane),
                }),
            };

            panes.push(clip);

            panes
        } else {
            panes
        };

        (
            Primitive::Group { primitives },
            if dragging.is_some() {
                MouseCursor::Grabbing
            } else {
                mouse_cursor
            },
        )
    }
}
