use super::Renderer;

use ggez::graphics::{DrawParam, Rect};
use iced::{slider, MouseCursor, Point, Rectangle};
use std::ops::RangeInclusive;

const RAIL: Rect = Rect {
    x: 98.0,
    y: 56.0,
    w: 1.0,
    h: 4.0,
};

const MARKER: Rect = Rect {
    x: RAIL.x + 28.0,
    y: RAIL.y,
    w: 16.0,
    h: 24.0,
};

impl slider::Renderer for Renderer<'_> {
    fn draw(
        &mut self,
        cursor_position: Point,
        bounds: Rectangle,
        state: &slider::State,
        range: RangeInclusive<f32>,
        value: f32,
    ) -> MouseCursor {
        let width = self.spritesheet.width() as f32;
        let height = self.spritesheet.height() as f32;

        self.sprites.add(DrawParam {
            src: Rect {
                x: RAIL.x / width,
                y: RAIL.y / height,
                w: RAIL.w / width,
                h: RAIL.h / height,
            },
            dest: ggez::mint::Point2 {
                x: bounds.x + MARKER.w as f32 / 2.0,
                y: bounds.y + 12.5,
            },
            scale: ggez::mint::Vector2 {
                x: bounds.width - MARKER.w as f32,
                y: 1.0,
            },
            ..DrawParam::default()
        });

        let (range_start, range_end) = range.into_inner();

        let marker_offset = (bounds.width - MARKER.w as f32)
            * ((value - range_start) / (range_end - range_start).max(1.0));

        let mouse_over = bounds.contains(cursor_position);
        let is_active = state.is_dragging() || mouse_over;

        self.sprites.add(DrawParam {
            src: Rect {
                x: (MARKER.x + (if is_active { MARKER.w } else { 0.0 }))
                    / width,
                y: MARKER.y / height,
                w: MARKER.w / width,
                h: MARKER.h / height,
            },
            dest: ggez::mint::Point2 {
                x: bounds.x + marker_offset.round(),
                y: bounds.y + (if state.is_dragging() { 2.0 } else { 0.0 }),
            },
            ..DrawParam::default()
        });

        if state.is_dragging() {
            MouseCursor::Grabbing
        } else if mouse_over {
            MouseCursor::Grab
        } else {
            MouseCursor::OutOfBounds
        }
    }
}
