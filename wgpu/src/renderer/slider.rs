use crate::{Primitive, Renderer};
use iced_native::{
    slider, Background, Color, Layout, Length, MouseCursor, Node, Point,
    Rectangle, Slider, Style,
};

const HANDLE_WIDTH: f32 = 8.0;
const HANDLE_HEIGHT: f32 = 22.0;

impl slider::Renderer for Renderer {
    fn node<Message>(&self, slider: &Slider<Message>) -> Node {
        let style = Style::default()
            .width(slider.width)
            .height(Length::Units(HANDLE_HEIGHT as u16))
            .min_width(Length::Units(100));

        Node::new(style)
    }

    fn draw<Message>(
        &mut self,
        slider: &Slider<Message>,
        layout: Layout<'_>,
        cursor_position: Point,
    ) -> Self::Output {
        let bounds = layout.bounds();

        let is_mouse_over = bounds.contains(cursor_position);

        let rail_y = bounds.y + (bounds.height / 2.0).round();

        let (rail_top, rail_bottom) = (
            Primitive::Quad {
                bounds: Rectangle {
                    x: bounds.x,
                    y: rail_y,
                    width: bounds.width,
                    height: 2.0,
                },
                background: Background::Color(Color {
                    r: 0.6,
                    g: 0.6,
                    b: 0.6,
                    a: 1.0,
                }),
                border_radius: 0,
            },
            Primitive::Quad {
                bounds: Rectangle {
                    x: bounds.x,
                    y: rail_y + 2.0,
                    width: bounds.width,
                    height: 2.0,
                },
                background: Background::Color(Color::WHITE),
                border_radius: 0,
            },
        );

        let (range_start, range_end) = slider.range.clone().into_inner();

        let handle_offset = (bounds.width - HANDLE_WIDTH)
            * ((slider.value - range_start)
                / (range_end - range_start).max(1.0));

        let (handle_border, handle) = (
            Primitive::Quad {
                bounds: Rectangle {
                    x: bounds.x + handle_offset.round() - 1.0,
                    y: rail_y - HANDLE_HEIGHT / 2.0 - 1.0,
                    width: HANDLE_WIDTH + 2.0,
                    height: HANDLE_HEIGHT + 2.0,
                },
                background: Background::Color(Color {
                    r: 0.6,
                    g: 0.6,
                    b: 0.6,
                    a: 1.0,
                }),
                border_radius: 5,
            },
            Primitive::Quad {
                bounds: Rectangle {
                    x: bounds.x + handle_offset.round(),
                    y: rail_y - HANDLE_HEIGHT / 2.0,
                    width: HANDLE_WIDTH,
                    height: HANDLE_HEIGHT,
                },
                background: Background::Color(if slider.state.is_dragging() {
                    Color {
                        r: 0.85,
                        g: 0.85,
                        b: 0.85,
                        a: 1.0,
                    }
                } else if is_mouse_over {
                    Color {
                        r: 0.9,
                        g: 0.9,
                        b: 0.9,
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
                border_radius: 4,
            },
        );

        (
            Primitive::Group {
                primitives: vec![rail_top, rail_bottom, handle_border, handle],
            },
            if slider.state.is_dragging() {
                MouseCursor::Grabbing
            } else if is_mouse_over {
                MouseCursor::Grab
            } else {
                MouseCursor::OutOfBounds
            },
        )
    }
}
