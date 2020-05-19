//! Allow your users to visually track the progress of a computation.
//!
//! A [`ProgressBar`] has a range of possible values and a current value,
//! as well as a length, height and style.
//!
//! [`ProgressBar`]: type.ProgressBar.html
use crate::{Backend, Primitive, Renderer};
use iced_native::mouse;
use iced_native::progress_bar;
use iced_native::{Color, Rectangle};

pub use iced_style::progress_bar::{Style, StyleSheet};

/// A bar that displays progress.
///
/// This is an alias of an `iced_native` progress bar with an
/// `iced_wgpu::Renderer`.
pub type ProgressBar<Backend> = iced_native::ProgressBar<Renderer<Backend>>;

impl<B> progress_bar::Renderer for Renderer<B>
where
    B: Backend,
{
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
