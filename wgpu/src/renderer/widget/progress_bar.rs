use crate::{Primitive, Renderer};
use iced_native::{progress_bar, Background, Color, MouseCursor, Rectangle};

impl progress_bar::Renderer for Renderer {
    const DEFAULT_HEIGHT: u16 = 30;

    fn draw(
        &self,
        bounds: Rectangle,
        range: std::ops::RangeInclusive<f32>,
        value: f32,
        background: Option<Background>,
        active_color: Option<Color>,
    ) -> Self::Output {
        let (range_start, range_end) = range.into_inner();
        let active_progress_width = bounds.width
            * ((value - range_start) / (range_end - range_start).max(1.0));

        let background = Primitive::Group {
            primitives: vec![Primitive::Quad {
                bounds: Rectangle { ..bounds },
                background: background
                    .unwrap_or(Background::Color([0.6, 0.6, 0.6].into()))
                    .into(),
                border_radius: 5,
                border_width: 0,
                border_color: Color::TRANSPARENT,
            }],
        };

        let active_progress = Primitive::Quad {
            bounds: Rectangle {
                width: active_progress_width,
                ..bounds
            },
            background: Background::Color(
                active_color.unwrap_or([0.0, 0.95, 0.0].into()),
            ),
            border_radius: 5,
            border_width: 0,
            border_color: Color::TRANSPARENT,
        };

        (
            Primitive::Group {
                primitives: vec![background, active_progress],
            },
            MouseCursor::OutOfBounds,
        )
    }
}
