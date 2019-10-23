use crate::{Primitive, Renderer};
use iced_native::{
    radio, text, Align, Background, Color, Column, Layout, Length, MouseCursor,
    Node, Point, Radio, Rectangle, Row, Text, Widget,
};

const SIZE: f32 = 28.0;
const DOT_SIZE: f32 = SIZE / 2.0;

impl radio::Renderer for Renderer {
    fn node<Message>(&self, radio: &Radio<Message>) -> Node {
        Row::<(), Self>::new()
            .spacing(15)
            .align_items(Align::Center)
            .push(
                Column::new()
                    .width(Length::Units(SIZE as u16))
                    .height(Length::Units(SIZE as u16)),
            )
            .push(Text::new(&radio.label))
            .node(self)
    }

    fn draw<Message>(
        &mut self,
        radio: &Radio<Message>,
        layout: Layout<'_>,
        cursor_position: Point,
    ) -> Self::Output {
        let bounds = layout.bounds();
        let mut children = layout.children();

        let radio_bounds = children.next().unwrap().bounds();
        let label_layout = children.next().unwrap();

        let (label, _) =
            text::Renderer::draw(self, &Text::new(&radio.label), label_layout);

        let is_mouse_over = bounds.contains(cursor_position);

        let (radio_border, radio_box) = (
            Primitive::Quad {
                bounds: radio_bounds,
                background: Background::Color(Color {
                    r: 0.6,
                    g: 0.6,
                    b: 0.6,
                    a: 1.0,
                }),
                border_radius: (SIZE / 2.0) as u16,
            },
            Primitive::Quad {
                bounds: Rectangle {
                    x: radio_bounds.x + 1.0,
                    y: radio_bounds.y + 1.0,
                    width: radio_bounds.width - 2.0,
                    height: radio_bounds.height - 2.0,
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
                border_radius: (SIZE / 2.0 - 1.0) as u16,
            },
        );

        (
            Primitive::Group {
                primitives: if radio.is_selected {
                    let radio_circle = Primitive::Quad {
                        bounds: Rectangle {
                            x: radio_bounds.x + DOT_SIZE / 2.0,
                            y: radio_bounds.y + DOT_SIZE / 2.0,
                            width: radio_bounds.width - DOT_SIZE,
                            height: radio_bounds.height - DOT_SIZE,
                        },
                        background: Background::Color(Color {
                            r: 0.30,
                            g: 0.30,
                            b: 0.30,
                            a: 1.0,
                        }),
                        border_radius: (DOT_SIZE / 2.0) as u16,
                    };

                    vec![radio_border, radio_box, radio_circle, label]
                } else {
                    vec![radio_border, radio_box, label]
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
