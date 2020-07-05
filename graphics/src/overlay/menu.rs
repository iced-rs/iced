use crate::backend::{self, Backend};
use crate::{Primitive, Renderer};
use iced_native::{
    mouse, overlay, Color, Font, HorizontalAlignment, Point, Rectangle,
    VerticalAlignment,
};

pub use iced_style::menu::Style;

impl<B> overlay::menu::Renderer for Renderer<B>
where
    B: Backend + backend::Text,
{
    type Style = Style;

    fn decorate(
        &mut self,
        bounds: Rectangle,
        _cursor_position: Point,
        style: &Style,
        (primitives, mouse_cursor): Self::Output,
    ) -> Self::Output {
        (
            Primitive::Group {
                primitives: vec![
                    Primitive::Quad {
                        bounds,
                        background: style.background,
                        border_color: style.border_color,
                        border_width: style.border_width,
                        border_radius: 0,
                    },
                    primitives,
                ],
            },
            mouse_cursor,
        )
    }

    fn draw<T: ToString>(
        &mut self,
        bounds: Rectangle,
        cursor_position: Point,
        options: &[T],
        hovered_option: Option<usize>,
        text_size: u16,
        padding: u16,
        style: &Style,
    ) -> Self::Output {
        use std::f32;

        let is_mouse_over = bounds.contains(cursor_position);

        let mut primitives = Vec::new();

        for (i, option) in options.iter().enumerate() {
            let is_selected = hovered_option == Some(i);

            let bounds = Rectangle {
                x: bounds.x,
                y: bounds.y
                    + ((text_size as usize + padding as usize * 2) * i) as f32,
                width: bounds.width,
                height: f32::from(text_size + padding * 2),
            };

            if is_selected {
                primitives.push(Primitive::Quad {
                    bounds,
                    background: style.selected_background,
                    border_color: Color::TRANSPARENT,
                    border_width: 0,
                    border_radius: 0,
                });
            }

            primitives.push(Primitive::Text {
                content: option.to_string(),
                bounds: Rectangle {
                    x: bounds.x + f32::from(padding),
                    y: bounds.center_y(),
                    width: f32::INFINITY,
                    ..bounds
                },
                size: f32::from(text_size),
                font: Font::Default,
                color: if is_selected {
                    style.selected_text_color
                } else {
                    style.text_color
                },
                horizontal_alignment: HorizontalAlignment::Left,
                vertical_alignment: VerticalAlignment::Center,
            });
        }

        (
            Primitive::Group { primitives },
            if is_mouse_over {
                mouse::Interaction::Pointer
            } else {
                mouse::Interaction::default()
            },
        )
    }
}
