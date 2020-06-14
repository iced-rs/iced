//! Display an interactive selector of a single value from a range of values.
//!
//! A [`Slider`] has some local [`State`].
//!
//! [`Slider`]: struct.Slider.html
//! [`State`]: struct.State.html
use crate::{Backend, Primitive, Renderer};
use iced_native::mouse;
use iced_native::slider;
use iced_native::{Background, Color, Point, Rectangle};

pub use iced_native::slider::State;
pub use iced_style::slider::{Handle, HandleShape, Style, StyleSheet};

/// An horizontal bar and a handle that selects a single value from a range of
/// values.
///
/// This is an alias of an `iced_native` slider with an `iced_wgpu::Renderer`.
pub type Slider<'a, T, Message, Backend> =
    iced_native::Slider<'a, T, Message, Renderer<Backend>>;

const HANDLE_HEIGHT: f32 = 22.0;

impl<B> slider::Renderer for Renderer<B>
where
    B: Backend,
{
    type Style = Box<dyn StyleSheet>;

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
        style_sheet: &Self::Style,
    ) -> Self::Output {
        let is_mouse_over = bounds.contains(cursor_position);

        let style = if is_dragging {
            style_sheet.dragging()
        } else if is_mouse_over {
            style_sheet.hovered()
        } else {
            style_sheet.active()
        };

        let rail_y = bounds.y + (bounds.height / 2.0).round();

        let (rail_top, rail_bottom) = (
            Primitive::Quad {
                bounds: Rectangle {
                    x: bounds.x,
                    y: rail_y,
                    width: bounds.width,
                    height: 2.0,
                },
                background: Background::Color(style.rail_colors.0),
                border_radius: 0,
                border_width: 0,
                border_color: Color::TRANSPARENT,
            },
            Primitive::Quad {
                bounds: Rectangle {
                    x: bounds.x,
                    y: rail_y + 2.0,
                    width: bounds.width,
                    height: 2.0,
                },
                background: Background::Color(style.rail_colors.1),
                border_radius: 0,
                border_width: 0,
                border_color: Color::TRANSPARENT,
            },
        );

        let (range_start, range_end) = range.into_inner();

        let (handle_width, handle_height, handle_border_radius) =
            match style.handle.shape {
                HandleShape::Circle { radius } => {
                    (f32::from(radius * 2), f32::from(radius * 2), radius)
                }
                HandleShape::Rectangle {
                    width,
                    border_radius,
                } => (f32::from(width), HANDLE_HEIGHT, border_radius),
            };

        let handle_offset = (bounds.width - handle_width)
            * ((value - range_start) / (range_end - range_start).max(1.0));

        let handle = Primitive::Quad {
            bounds: Rectangle {
                x: bounds.x + handle_offset.round(),
                y: rail_y - handle_height / 2.0,
                width: handle_width,
                height: handle_height,
            },
            background: Background::Color(style.handle.color),
            border_radius: handle_border_radius,
            border_width: style.handle.border_width,
            border_color: style.handle.border_color,
        };

        (
            Primitive::Group {
                primitives: vec![rail_top, rail_bottom, handle],
            },
            if is_dragging {
                mouse::Interaction::Grabbing
            } else if is_mouse_over {
                mouse::Interaction::Grab
            } else {
                mouse::Interaction::default()
            },
        )
    }
}
