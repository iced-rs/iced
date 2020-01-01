use crate::{Primitive, Renderer};
use iced_native::{progressbar, Background, Color, MouseCursor, Rectangle};

impl progressbar::Renderer for Renderer {
    fn height(&self) -> u32 {
        30
    }

    fn draw(
        &self,
        bounds: Rectangle,
        range: std::ops::RangeInclusive<f32>,
        value: f32,
    ) -> Self::Output {
        let (range_start, range_end) = range.into_inner();
        let active_progress_width = bounds.width
            * ((value - range_start) / (range_end - range_start).max(1.0));

        let background = Primitive::Group {
            primitives: vec![Primitive::Quad {
                bounds: Rectangle { ..bounds },
                background: Color::from_rgb(0.6, 0.6, 0.6).into(),
                border_radius: 5,
            }],
        };

        let active_progress = Primitive::Quad {
            bounds: Rectangle {
                width: active_progress_width,
                ..bounds
            },
            background: Background::Color([0.0, 0.95, 0.0].into()),
            border_radius: 4,
        };

        (
            Primitive::Group {
                primitives: vec![background, active_progress],
            },
            MouseCursor::OutOfBounds,
        )
    }
}
