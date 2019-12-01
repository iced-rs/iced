use crate::{Primitive, Renderer};
use iced_native::{button, Background, Color, MouseCursor, Point, Rectangle};

impl button::Renderer for Renderer {
    fn draw(
        &mut self,
        bounds: Rectangle,
        cursor_position: Point,
        is_pressed: bool,
        background: Option<Background>,
        border_radius: u16,
        (content, _): Self::Output,
    ) -> Self::Output {
        let is_mouse_over = bounds.contains(cursor_position);

        // TODO: Render proper shadows
        // TODO: Make hovering and pressed styles configurable
        let shadow_offset = if is_mouse_over {
            if is_pressed {
                0.0
            } else {
                2.0
            }
        } else {
            1.0
        };

        (
            match background {
                None => content,
                Some(background) => Primitive::Group {
                    primitives: vec![
                        Primitive::Quad {
                            bounds: Rectangle {
                                x: bounds.x + 1.0,
                                y: bounds.y + shadow_offset,
                                ..bounds
                            },
                            background: Background::Color(
                                [0.0, 0.0, 0.0, 0.5].into(),
                            ),
                            border_radius,
                            border_color: Color::BLACK,
                            border_width: 0,
                        },
                        Primitive::Quad {
                            bounds,
                            background,
                            border_radius,
                            border_color: Color::BLACK,
                            border_width: 0,
                        },
                        content,
                    ],
                },
            },
            if is_mouse_over {
                MouseCursor::Pointer
            } else {
                MouseCursor::OutOfBounds
            },
        )
    }
}
