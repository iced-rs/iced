use crate::{Primitive, Renderer};
use iced_native::{
    layout, radio, text, Background, Layout, MouseCursor, Point, Radio,
    Rectangle, Text,
};

const SIZE: f32 = 28.0;
const DOT_SIZE: f32 = SIZE / 2.0;

impl radio::Renderer for Renderer {
    fn layout<Message>(
        &self,
        radio: &Radio<Message>,
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
        radio: &Radio<Message>,
        layout: &Layout,
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
                background: Background::Color([0.6, 0.6, 0.6].into()),
                border_radius: (SIZE / 2.0) as u16,
            },
            Primitive::Quad {
                bounds: Rectangle {
                    x: radio_bounds.x + 1.0,
                    y: radio_bounds.y + 1.0,
                    width: radio_bounds.width - 2.0,
                    height: radio_bounds.height - 2.0,
                },
                background: Background::Color(
                    if is_mouse_over {
                        [0.90, 0.90, 0.90]
                    } else {
                        [0.95, 0.95, 0.95]
                    }
                    .into(),
                ),
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
                        background: Background::Color([0.3, 0.3, 0.3].into()),
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
