use crate::backend::Backend;
use crate::{Primitive, Renderer};
use iced_native::{
    layer, mouse, Background, Color, Font, HorizontalAlignment, Point,
    Rectangle, VerticalAlignment,
};

impl<B> layer::menu::Renderer for Renderer<B>
where
    B: Backend,
{
    fn decorate(
        &mut self,
        bounds: Rectangle,
        _cursor_position: Point,
        (primitives, mouse_cursor): Self::Output,
    ) -> Self::Output {
        (
            Primitive::Group {
                primitives: vec![
                    Primitive::Quad {
                        bounds,
                        background: Background::Color(
                            [0.87, 0.87, 0.87].into(),
                        ),
                        border_color: [0.7, 0.7, 0.7].into(),
                        border_width: 1,
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
                    background: Background::Color([0.4, 0.4, 1.0].into()),
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
                    Color::WHITE
                } else {
                    Color::BLACK
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
