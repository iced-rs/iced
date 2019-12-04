use crate::{Primitive, Renderer};
use iced_native::{slider, Background, Color, MouseCursor, Point, Rectangle};

const HANDLE_WIDTH: f32 = 8.0;
const HANDLE_HEIGHT: f32 = 22.0;

impl slider::Renderer for Renderer {
    fn height(&self) -> u32 {
        30
    }

    fn draw(
        &mut self,
        bounds: Rectangle,
        cursor_position: Point,
        range: std::ops::RangeInclusive<f32>,
        value: f32,
        is_dragging: bool,
    ) -> Self::Output {
        let is_mouse_over = bounds.contains(cursor_position);

        let rail_y = bounds.y + (bounds.height / 2.0).round();

        let (rail_top, rail_bottom) = (
            Primitive::Quad {
                bounds: Rectangle {
                    x: bounds.x,
                    y: rail_y,
                    width: bounds.width,
                    height: 2.0,
                },
                background: Color::from_rgb(0.6, 0.6, 0.6).into(),
                border_radius: 0,
            },
            Primitive::Quad {
                bounds: Rectangle {
                    x: bounds.x,
                    y: rail_y + 2.0,
                    width: bounds.width,
                    height: 2.0,
                },
                background: Background::Color(Color::WHITE),
                border_radius: 0,
            },
        );

        let (range_start, range_end) = range.into_inner();

        let handle_offset = (bounds.width - HANDLE_WIDTH)
            * ((value - range_start) / (range_end - range_start).max(1.0));

        let (handle_border, handle) = (
            Primitive::Quad {
                bounds: Rectangle {
                    x: bounds.x + handle_offset.round() - 1.0,
                    y: rail_y - HANDLE_HEIGHT / 2.0 - 1.0,
                    width: HANDLE_WIDTH + 2.0,
                    height: HANDLE_HEIGHT + 2.0,
                },
                background: Color::from_rgb(0.6, 0.6, 0.6).into(),
                border_radius: 5,
            },
            Primitive::Quad {
                bounds: Rectangle {
                    x: bounds.x + handle_offset.round(),
                    y: rail_y - HANDLE_HEIGHT / 2.0,
                    width: HANDLE_WIDTH,
                    height: HANDLE_HEIGHT,
                },
                background: Background::Color(
                    if is_dragging {
                        [0.85, 0.85, 0.85]
                    } else if is_mouse_over {
                        [0.90, 0.90, 0.90]
                    } else {
                        [0.95, 0.95, 0.95]
                    }
                    .into(),
                ),
                border_radius: 4,
            },
        );

        (
            Primitive::Group {
                primitives: vec![rail_top, rail_bottom, handle_border, handle],
            },
            if is_dragging {
                MouseCursor::Grabbing
            } else if is_mouse_over {
                MouseCursor::Grab
            } else {
                MouseCursor::OutOfBounds
            },
        )
    }
}
