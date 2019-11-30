use crate::{Primitive, Renderer};
use iced_native::{
    scrollable, Background, MouseCursor, Point, Rectangle, Vector, Color,
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

    fn draw(
        &mut self,
        state: &scrollable::State,
        bounds: Rectangle,
        content_bounds: Rectangle,
        is_mouse_over: bool,
        is_mouse_over_scrollbar: bool,
        offset: u32,
        (content, mouse_cursor): Self::Output,
    ) -> Self::Output {
        let is_content_overflowing = content_bounds.height > bounds.height;
        let scrollbar_bounds = scrollbar_bounds(bounds);

        let clip = Primitive::Clip {
            bounds,
            offset: Vector::new(0, offset),
            content: Box::new(content),
        };

        (
            if is_content_overflowing
                && (is_mouse_over || state.is_scrollbar_grabbed())
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
                    border_color: Color::BLACK,
                    border_width: 0,
                };

                if is_mouse_over_scrollbar || state.is_scrollbar_grabbed() {
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
                        border_color: Color::BLACK,
                        border_width: 0,
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
            if is_mouse_over_scrollbar || state.is_scrollbar_grabbed() {
                MouseCursor::Idle
            } else {
                mouse_cursor
            },
        )
    }
}
