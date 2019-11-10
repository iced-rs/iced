use crate::{Primitive, Renderer};
use iced_native::{
    checkbox, layout, text, text::HorizontalAlignment, text::VerticalAlignment,
    Background, Checkbox, Layout, MouseCursor, Point, Rectangle, Text,
};

const SIZE: f32 = 28.0;

impl checkbox::Renderer for Renderer {
    fn layout<Message>(
        &self,
        checkbox: &Checkbox<Message>,
        limits: &layout::Limits,
    ) -> Layout {
        // TODO
        Layout::new(Rectangle {
            x: 0.0,
            y: 0.0,
            width: 0.0,
            height: 0.0,
        })
    }

    fn draw<Message>(
        &mut self,
        checkbox: &Checkbox<Message>,
        layout: &Layout,
        cursor_position: Point,
    ) -> Self::Output {
        let bounds = layout.bounds();
        let mut children = layout.children();

        let checkbox_layout = children.next().unwrap();
        let label_layout = children.next().unwrap();
        let checkbox_bounds = checkbox_layout.bounds();

        let (label, _) = text::Renderer::draw(
            self,
            &Text::new(&checkbox.label),
            label_layout,
        );

        let is_mouse_over = bounds.contains(cursor_position);

        let (checkbox_border, checkbox_box) = (
            Primitive::Quad {
                bounds: checkbox_bounds,
                background: Background::Color([0.6, 0.6, 0.6].into()),
                border_radius: 6,
            },
            Primitive::Quad {
                bounds: Rectangle {
                    x: checkbox_bounds.x + 1.0,
                    y: checkbox_bounds.y + 1.0,
                    width: checkbox_bounds.width - 2.0,
                    height: checkbox_bounds.height - 2.0,
                },
                background: Background::Color(
                    if is_mouse_over {
                        [0.90, 0.90, 0.90]
                    } else {
                        [0.95, 0.95, 0.95]
                    }
                    .into(),
                ),
                border_radius: 5,
            },
        );

        (
            Primitive::Group {
                primitives: if checkbox.is_checked {
                    // TODO: Draw an actual icon
                    let (check, _) = text::Renderer::draw(
                        self,
                        &Text::new("X")
                            .horizontal_alignment(HorizontalAlignment::Center)
                            .vertical_alignment(VerticalAlignment::Center),
                        checkbox_layout,
                    );

                    vec![checkbox_border, checkbox_box, check, label]
                } else {
                    vec![checkbox_border, checkbox_box, label]
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
