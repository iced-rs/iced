use crate::{Primitive, Renderer};
use iced_native::{
    checkbox, text, text::HorizontalAlignment, text::VerticalAlignment, Align,
    Background, Checkbox, Color, Column, Layout, Length, MouseCursor, Node,
    Point, Rectangle, Row, Text, Widget,
};

const SIZE: f32 = 28.0;

impl checkbox::Renderer for Renderer {
    fn node<Message>(&self, checkbox: &Checkbox<Message>) -> Node {
        Row::<(), Self>::new()
            .spacing(15)
            .align_items(Align::Center)
            .push(
                Column::new()
                    .width(Length::Units(SIZE as u16))
                    .height(Length::Units(SIZE as u16)),
            )
            .push(Text::new(&checkbox.label))
            .node(self)
    }

    fn draw<Message>(
        &mut self,
        checkbox: &Checkbox<Message>,
        layout: Layout<'_>,
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
                background: Background::Color(Color {
                    r: 0.6,
                    g: 0.6,
                    b: 0.6,
                    a: 1.0,
                }),
                border_radius: 6,
            },
            Primitive::Quad {
                bounds: Rectangle {
                    x: checkbox_bounds.x + 1.0,
                    y: checkbox_bounds.y + 1.0,
                    width: checkbox_bounds.width - 2.0,
                    height: checkbox_bounds.height - 2.0,
                },
                background: Background::Color(if is_mouse_over {
                    Color {
                        r: 0.90,
                        g: 0.90,
                        b: 0.90,
                        a: 1.0,
                    }
                } else {
                    Color {
                        r: 0.95,
                        g: 0.95,
                        b: 0.95,
                        a: 1.0,
                    }
                }),
                border_radius: 6,
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
