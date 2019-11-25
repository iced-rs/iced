use crate::{Primitive, Renderer};
use iced_native::{
    scrollable, Background, MouseCursor, Point, Rectangle, ScrollbarGrab,
    Vector,
};

const SCROLLBAR_WIDTH: u16 = 10;
const SCROLLBAR_MARGIN: u16 = 2;

fn background_bounds(bounds: Rectangle) -> Rectangle {
    Rectangle {
        x: bounds.x + bounds.width
            - f32::from(SCROLLBAR_WIDTH + 2 * SCROLLBAR_MARGIN),
        y: bounds.y,
        width: f32::from(SCROLLBAR_WIDTH + 2 * SCROLLBAR_MARGIN),
        height: bounds.height,
    }
}

fn scroller_bounds(
    bounds: Rectangle,
    content_bounds: Rectangle,
    background_bounds: Rectangle,
    offset: u32,
) -> Rectangle {
    let ratio = bounds.height / content_bounds.height;
    let scrollbar_height = bounds.height * ratio;
    let y_offset = offset as f32 * ratio;

    Rectangle {
        x: background_bounds.x + f32::from(SCROLLBAR_MARGIN),
        y: background_bounds.y + y_offset,
        width: background_bounds.width - f32::from(2 * SCROLLBAR_MARGIN),
        height: scrollbar_height,
    }
}

impl scrollable::Renderer for Renderer {
    fn scrollbar_grab(
        &self,
        bounds: Rectangle,
        content_bounds: Rectangle,
        offset: u32,
        cursor_position: Point,
    ) -> Option<(ScrollbarGrab, Rectangle)> {
        let background_bounds = background_bounds(bounds);
        if content_bounds.height > bounds.height
            && background_bounds.contains(cursor_position)
        {
            let scroller_bounds = scroller_bounds(
                bounds,
                content_bounds,
                background_bounds,
                offset,
            );

            let scrollbar_grab = if scroller_bounds.contains(cursor_position) {
                ScrollbarGrab::Scroller
            } else {
                ScrollbarGrab::Background
            };

            Some((scrollbar_grab, scroller_bounds))
        } else {
            None
        }
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
        let background_bounds = background_bounds(bounds);

        let clip = Primitive::Clip {
            bounds,
            offset: Vector::new(0, offset),
            content: Box::new(content),
        };

        (
            if is_content_overflowing
                && (is_mouse_over || state.currently_grabbed())
            {
                let scroller_bounds = scroller_bounds(
                    bounds,
                    content_bounds,
                    background_bounds,
                    offset,
                );
                let scrollbar = Primitive::Quad {
                    bounds: scroller_bounds,
                    background: Background::Color([0.0, 0.0, 0.0, 0.7].into()),
                    border_radius: 5,
                };

                if is_mouse_over_scrollbar || state.currently_grabbed() {
                    let scrollbar_background = Primitive::Quad {
                        bounds: Rectangle {
                            x: background_bounds.x
                                + f32::from(SCROLLBAR_MARGIN),
                            width: background_bounds.width
                                - f32::from(2 * SCROLLBAR_MARGIN),
                            ..background_bounds
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
            if is_mouse_over_scrollbar || state.currently_grabbed() {
                MouseCursor::Idle
            } else {
                mouse_cursor
            },
        )
    }
}
