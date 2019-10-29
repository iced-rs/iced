use crate::{Primitive, Renderer};
use iced_native::{
    scrollable, Background, Color, Layout, Point, Rectangle, Scrollable, Widget,
};

impl scrollable::Renderer for Renderer {
    fn draw<Message>(
        &mut self,
        scrollable: &Scrollable<'_, Message, Self>,
        bounds: Rectangle,
        content: Layout<'_>,
        cursor_position: Point,
    ) -> Self::Output {
        let is_mouse_over = bounds.contains(cursor_position);
        let content_bounds = content.bounds();

        let offset = scrollable.state.offset(bounds, content_bounds);

        let cursor_position = if bounds.contains(cursor_position) {
            Point::new(cursor_position.x, cursor_position.y + offset as f32)
        } else {
            Point::new(cursor_position.x, -1.0)
        };

        let (content, mouse_cursor) =
            scrollable.content.draw(self, content, cursor_position);

        let primitive = Primitive::Scrollable {
            bounds,
            offset,
            content: Box::new(content),
        };

        (
            if is_mouse_over && content_bounds.height > bounds.height {
                let ratio = bounds.height / content_bounds.height;
                let scrollbar_height = bounds.height * ratio;
                let y_offset = offset as f32 * ratio;

                let scrollbar = Primitive::Quad {
                    bounds: Rectangle {
                        x: bounds.x + bounds.width - 12.0,
                        y: bounds.y + y_offset,
                        width: 10.0,
                        height: scrollbar_height,
                    },
                    background: Background::Color(Color {
                        r: 0.0,
                        g: 0.0,
                        b: 0.0,
                        a: 0.7,
                    }),
                    border_radius: 5,
                };
                Primitive::Group {
                    primitives: vec![primitive, scrollbar],
                }
            } else {
                primitive
            },
            mouse_cursor,
        )
    }
}
