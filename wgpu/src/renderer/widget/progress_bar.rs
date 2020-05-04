use crate::{progress_bar::StyleSheet, Primitive, Renderer};
use iced_native::{mouse, progress_bar, Color, Rectangle};

impl progress_bar::Renderer for Renderer {
    type Style = Box<dyn StyleSheet>;

    const DEFAULT_HEIGHT: u16 = 30;

    fn draw(
        &self,
        bounds: Rectangle,
        range: std::ops::RangeInclusive<f32>,
        value: f32,
        style_sheet: &Self::Style,
    ) -> Self::Output {
        let style = style_sheet.style();

        let (range_start, range_end) = range.into_inner();
        let active_progress_width = bounds.width
            * ((value - range_start) / (range_end - range_start).max(1.0));

        let background = Primitive::Group {
            primitives: vec![Primitive::Quad {
                bounds: Rectangle { ..bounds },
                background: style.background,
                border_radius: style.border_radius,
                border_width: 0,
                border_color: Color::TRANSPARENT,
            }],
        };

        (
            if active_progress_width > 0.0 {
                let bar = Primitive::Quad {
                    bounds: Rectangle {
                        width: active_progress_width,
                        ..bounds
                    },
                    background: style.bar,
                    border_radius: style.border_radius,
                    border_width: 0,
                    border_color: Color::TRANSPARENT,
                };

                Primitive::Group {
                    primitives: vec![background, bar],
                }
            } else {
                background
            },
            mouse::Interaction::default(),
        )
    }
}
