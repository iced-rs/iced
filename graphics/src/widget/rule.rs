//! Display a horizontal or vertical rule for dividing content.

use crate::{Backend, Primitive, Renderer};
use iced_native::mouse;
use iced_native::rule;
use iced_native::{Background, Color, Rectangle};

pub use iced_style::rule::{Style, StyleSheet};

/// Display a horizontal or vertical rule for dividing content.
///
/// This is an alias of an `iced_native` rule with an `iced_graphics::Renderer`.
pub type Rule<Backend> = iced_native::Rule<Renderer<Backend>>;

impl<B> rule::Renderer for Renderer<B>
where
    B: Backend,
{
    type Style = Box<dyn StyleSheet>;

    fn draw(
        &mut self,
        bounds: Rectangle,
        style_sheet: &Self::Style,
        is_horizontal: bool,
    ) -> Self::Output {
        let style = style_sheet.style();

        let line = if is_horizontal {
            let line_y = (bounds.y + (bounds.height / 2.0)
                - (style.width as f32 / 2.0))
                .round();

            let (line_x, line_width) = if style.fill_percent >= 100 {
                (bounds.x, bounds.width)
            } else {
                let percent_width =
                    (bounds.width * style.fill_percent as f32 / 100.0).round();
                (
                    bounds.x + ((bounds.width - percent_width) / 2.0).round(),
                    percent_width,
                )
            };

            Primitive::Quad {
                bounds: Rectangle {
                    x: line_x,
                    y: line_y,
                    width: line_width,
                    height: style.width as f32,
                },
                background: Background::Color(style.color),
                border_radius: style.radius,
                border_width: 0,
                border_color: Color::TRANSPARENT,
            }
        } else {
            let line_x = (bounds.x + (bounds.width / 2.0)
                - (style.width as f32 / 2.0))
                .round();

            let (line_y, line_height) = if style.fill_percent >= 100 {
                (bounds.y, bounds.height)
            } else {
                let percent_height =
                    (bounds.height * style.fill_percent as f32 / 100.0).round();
                (
                    bounds.y + ((bounds.height - percent_height) / 2.0).round(),
                    percent_height,
                )
            };

            Primitive::Quad {
                bounds: Rectangle {
                    x: line_x,
                    y: line_y,
                    width: style.width as f32,
                    height: line_height,
                },
                background: Background::Color(style.color),
                border_radius: style.radius,
                border_width: 0,
                border_color: Color::TRANSPARENT,
            }
        };

        (line, mouse::Interaction::default())
    }
}
