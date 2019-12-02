use crate::{Primitive, Renderer};
use iced_native::{scrollable, Background, MouseCursor, Rectangle, Vector};

const SCROLLBAR_WIDTH: u16 = 10;
const SCROLLBAR_MARGIN: u16 = 2;

impl scrollable::Renderer for Renderer {
    fn scrollbar_bounds(bounds: Rectangle) -> Rectangle {
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
        scrollbar_bounds: Rectangle,
        offset: u32,
    ) -> Rectangle {
        let ratio = bounds.height / content_bounds.height;
        let scrollbar_height = bounds.height * ratio;
        let y_offset = offset as f32 * ratio;
        Rectangle {
            x: scrollbar_bounds.x + f32::from(SCROLLBAR_MARGIN),
            y: scrollbar_bounds.y + y_offset,
            width: scrollbar_bounds.width - f32::from(2 * SCROLLBAR_MARGIN),
            height: scrollbar_height,
        }
    }

    fn draw(
        &mut self,
        state: &scrollable::State,
        bounds: Rectangle,
        content_bounds: Rectangle,
        is_mouse_over: bool,
        is_mouse_over_scrollbar: bool,
        scrollbar_bounds: Rectangle,
        scroller_bounds: Rectangle,
        offset: u32,
        (content, mouse_cursor): Self::Output,
    ) -> Self::Output {
        let is_content_overflowing = content_bounds.height > bounds.height;

        let clip = Primitive::Clip {
            bounds,
            offset: Vector::new(0, offset),
            content: Box::new(content),
        };

        (
            if is_content_overflowing
                && (is_mouse_over || state.is_scroller_grabbed())
            {
                let scrollbar = Primitive::Quad {
                    bounds: scroller_bounds,
                    background: Background::Color([0.0, 0.0, 0.0, 0.7].into()),
                    border_radius: 5,
                };

                if is_mouse_over_scrollbar || state.is_scroller_grabbed() {
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
            if is_mouse_over_scrollbar || state.is_scroller_grabbed() {
                MouseCursor::Idle
            } else {
                mouse_cursor
            },
        )
    }
}
