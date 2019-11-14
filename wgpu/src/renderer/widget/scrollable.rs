use crate::{Primitive, Renderer};
use iced_native::{
    scrollable, Background, Layout, MouseCursor, Point, Rectangle, Scrollable,
    Vector, Widget,
};

const SCROLLBAR_WIDTH: u16 = 10;
const SCROLLBAR_MARGIN: u16 = 2;

fn scrollbar_bounds(bounds: Rectangle) -> Rectangle {
    Rectangle {
        x: bounds.x + bounds.width
            - f32::from(SCROLLBAR_WIDTH + 2 * SCROLLBAR_MARGIN),
        y: bounds.y,
        width: f32::from(SCROLLBAR_WIDTH + 2 * SCROLLBAR_MARGIN),
        height: bounds.height,
    }
}

impl scrollable::Renderer for Renderer {
    fn is_mouse_over_scrollbar(
        &self,
        bounds: Rectangle,
        content_bounds: Rectangle,
        cursor_position: Point,
    ) -> bool {
        content_bounds.height > bounds.height
            && scrollbar_bounds(bounds).contains(cursor_position)
    }

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
        let is_content_overflowing = content_bounds.height > bounds.height;
        let scrollbar_bounds = scrollbar_bounds(bounds);
        let is_mouse_over_scrollbar = self.is_mouse_over_scrollbar(
            bounds,
            content_bounds,
            cursor_position,
        );

        let cursor_position = if is_mouse_over && !is_mouse_over_scrollbar {
            Point::new(cursor_position.x, cursor_position.y + offset as f32)
        } else {
            Point::new(cursor_position.x, -1.0)
        };

        let (content, mouse_cursor) =
            scrollable.content.draw(self, content, cursor_position);

        let clip = Primitive::Clip {
            bounds,
            offset: Vector::new(0, offset),
            content: Box::new(content),
        };

        (
            if is_content_overflowing
                && (is_mouse_over || scrollable.state.is_scrollbar_grabbed())
            {
                let ratio = bounds.height / content_bounds.height;
                let scrollbar_height = bounds.height * ratio;
                let y_offset = offset as f32 * ratio;

                let scrollbar = Primitive::Quad {
                    bounds: Rectangle {
                        x: scrollbar_bounds.x + f32::from(SCROLLBAR_MARGIN),
                        y: scrollbar_bounds.y + y_offset,
                        width: scrollbar_bounds.width
                            - f32::from(2 * SCROLLBAR_MARGIN),
                        height: scrollbar_height,
                    },
                    background: Background::Color([0.0, 0.0, 0.0, 0.7].into()),
                    border_radius: 5,
                };

                if is_mouse_over_scrollbar
                    || scrollable.state.is_scrollbar_grabbed()
                {
                    let scrollbar_background = Primitive::Quad {
                        bounds: Rectangle {
                            x: scrollbar_bounds.x + f32::from(SCROLLBAR_MARGIN),
                            width: scrollbar_bounds.width
                                - f32::from(2 * SCROLLBAR_MARGIN),
                            ..scrollbar_bounds
                        },
                        background: Background::Color(
                            [0.0, 0.0, 0.0, 0.3].into(),
                        ),
                        border_radius: 5,
                    };

                    Primitive::Group {
                        primitives: vec![clip, scrollbar_background, scrollbar],
                    }
                } else {
                    Primitive::Group {
                        primitives: vec![clip, scrollbar],
                    }
                }
            } else {
                clip
            },
            if is_mouse_over_scrollbar
                || scrollable.state.is_scrollbar_grabbed()
            {
                MouseCursor::Idle
            } else {
                mouse_cursor
            },
        )
    }
}
