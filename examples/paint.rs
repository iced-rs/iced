//! This example showcases a simple native custom widget that renders arbitrary
//! path with `lyon`.
mod bezier {
    // For now, to implement a custom native widget you will need to add
    // `iced_native` and `iced_wgpu` to your dependencies.
    //
    // Then, you simply need to define your widget type and implement the
    // `iced_native::Widget` trait with the `iced_wgpu::Renderer`.
    //
    // Of course, you can choose to make the implementation renderer-agnostic,
    // if you wish to, by creating your own `Renderer` trait, which could be
    // implemented by `iced_wgpu` and other renderers.
    use iced_native::{
        input, layout, Clipboard, Element, Event, Hasher, Layout, Length,
        MouseCursor, Point, Size, Vector, Widget,
    };
    use iced_wgpu::{
        triangle::{Mesh2D, Vertex2D},
        Primitive, Renderer,
    };
    use lyon::tessellation::{
        basic_shapes, BuffersBuilder, StrokeAttributes, StrokeOptions,
        StrokeTessellator, VertexBuffers,
    };
    use std::sync::Arc;

    pub struct Bezier<'a, Message> {
        pending_points: &'a [Point],
        // [from, to, ctrl]
        bezier_points: &'a [[Point; 3]],
        on_click: Box<dyn Fn(Point) -> Message>,
    }

    impl<'a, Message> Bezier<'a, Message> {
        pub fn new<F>(
            bezier_points: &'a [[Point; 3]],
            pending_points: &'a [Point],
            on_click: F,
        ) -> Self
        where
            F: 'static + Fn(Point) -> Message,
        {
            assert!(pending_points.len() < 3);

            Self {
                bezier_points,
                pending_points,
                on_click: Box::new(on_click),
            }
        }
    }

    impl<'a, Message> Widget<Message, Renderer> for Bezier<'a, Message> {
        fn width(&self) -> Length {
            Length::Fill
        }

        fn height(&self) -> Length {
            Length::Fill
        }

        fn layout(
            &self,
            _renderer: &Renderer,
            limits: &layout::Limits,
        ) -> layout::Node {
            let size = limits
                .height(Length::Fill)
                .width(Length::Fill)
                .resolve(Size::ZERO);
            layout::Node::new(size)
        }

        fn draw(
            &self,
            _renderer: &mut Renderer,
            layout: Layout<'_>,
            cursor_position: Point,
        ) -> (Primitive, MouseCursor) {
            let mut buffer: VertexBuffers<Vertex2D, u16> = VertexBuffers::new();
            let mut path_builder = lyon::path::Path::builder();

            let bounds = layout.bounds();

            // Draw rectangle border with lyon.
            basic_shapes::stroke_rectangle(
                &lyon::math::Rect::new(
                    lyon::math::Point::new(bounds.x + 0.5, bounds.y + 0.5),
                    lyon::math::Size::new(
                        bounds.width - 1.0,
                        bounds.height - 1.0,
                    ),
                ),
                &StrokeOptions::default().with_line_width(1.0),
                &mut BuffersBuilder::new(
                    &mut buffer,
                    |pos: lyon::math::Point, _: StrokeAttributes| Vertex2D {
                        position: pos.to_array(),
                        color: [0.0, 0.0, 0.0, 1.0],
                    },
                ),
            )
            .unwrap();

            for pts in self.bezier_points {
                path_builder.move_to(lyon::math::Point::new(
                    pts[0].x + bounds.x,
                    pts[0].y + bounds.y,
                ));

                path_builder.quadratic_bezier_to(
                    lyon::math::Point::new(
                        pts[2].x + bounds.x,
                        pts[2].y + bounds.y,
                    ),
                    lyon::math::Point::new(
                        pts[1].x + bounds.x,
                        pts[1].y + bounds.y,
                    ),
                );
            }

            match self.pending_points.len() {
                0 => {}
                1 => {
                    path_builder.move_to(lyon::math::Point::new(
                        self.pending_points[0].x + bounds.x,
                        self.pending_points[0].y + bounds.y,
                    ));
                    path_builder.line_to(lyon::math::Point::new(
                        cursor_position.x,
                        cursor_position.y,
                    ));
                }
                2 => {
                    path_builder.move_to(lyon::math::Point::new(
                        self.pending_points[0].x + bounds.x,
                        self.pending_points[0].y + bounds.y,
                    ));
                    path_builder.quadratic_bezier_to(
                        lyon::math::Point::new(
                            cursor_position.x,
                            cursor_position.y,
                        ),
                        lyon::math::Point::new(
                            self.pending_points[1].x + bounds.x,
                            self.pending_points[1].y + bounds.y,
                        ),
                    );
                }
                _ => {
                    unreachable!();
                }
            }

            let mut tessellator = StrokeTessellator::new();

            // Draw strokes with lyon.
            tessellator
                .tessellate(
                    &path_builder.build(),
                    &StrokeOptions::default().with_line_width(3.0),
                    &mut BuffersBuilder::new(
                        &mut buffer,
                        |pos: lyon::math::Point, _: StrokeAttributes| {
                            Vertex2D {
                                position: pos.to_array(),
                                color: [0.0, 0.0, 0.0, 1.0],
                            }
                        },
                    ),
                )
                .unwrap();

            (
                Primitive::Clip {
                    bounds,
                    offset: Vector::new(0, 0),
                    content: Box::new(Primitive::Mesh2D(Arc::new(Mesh2D {
                        vertices: buffer.vertices,
                        indices: buffer.indices,
                    }))),
                },
                MouseCursor::OutOfBounds,
            )
        }

        fn hash_layout(&self, _state: &mut Hasher) {}

        fn on_event(
            &mut self,
            event: Event,
            layout: Layout<'_>,
            cursor_position: Point,
            messages: &mut Vec<Message>,
            _renderer: &Renderer,
            _clipboard: Option<&dyn Clipboard>,
        ) {
            let bounds = layout.bounds();
            match event {
                Event::Mouse(input::mouse::Event::Input { state, .. }) => {
                    if state == input::ButtonState::Pressed
                        && bounds.contains(cursor_position)
                    {
                        messages.push((self.on_click)(Point::new(
                            cursor_position.x - bounds.x,
                            cursor_position.y - bounds.y,
                        )));
                    }
                }
                _ => {}
            }
        }
    }

    impl<'a, Message> Into<Element<'a, Message, Renderer>> for Bezier<'a, Message>
    where
        Message: 'static,
    {
        fn into(self) -> Element<'a, Message, Renderer> {
            Element::new(self)
        }
    }
}

use bezier::Bezier;
use iced::{
    button, Align, Button, Color, Column, Container, Element, Length, Sandbox,
    Settings, Text,
};
use iced_native::Point;

pub fn main() {
    Example::run(Settings::default())
}

#[derive(Default)]
struct Example {
    bezier_points: Vec<[Point; 3]>,
    pending_points: Vec<Point>,
    button_state: button::State,
}

#[derive(Debug, Clone, Copy)]
enum Message {
    AddPoint(Point),
    Clear,
}

impl Sandbox for Example {
    type Message = Message;

    fn new() -> Self {
        Example::default()
    }

    fn title(&self) -> String {
        String::from("Bezier tool - Iced")
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::AddPoint(point) => {
                self.pending_points.push(point);
                if self.pending_points.len() == 3 {
                    self.bezier_points.push([
                        self.pending_points[0],
                        self.pending_points[1],
                        self.pending_points[2],
                    ]);
                    self.pending_points.clear();
                }
            }
            Message::Clear => {
                self.bezier_points.clear();
                self.pending_points.clear();
            }
        }
    }

    fn view(&mut self) -> Element<Message> {
        let content = Column::new()
            .padding(20)
            .spacing(20)
            .align_items(Align::Center)
            .push(
                Text::new("Bezier tool example")
                    .width(Length::Shrink)
                    .size(50),
            )
            .push(Bezier::new(
                self.bezier_points.as_slice(),
                self.pending_points.as_slice(),
                Message::AddPoint,
            ))
            .push(
                Button::new(&mut self.button_state, Text::new("Clear"))
                    .padding(8)
                    .background(Color::from_rgb(0.5, 0.5, 0.5))
                    .border_radius(4)
                    .on_press(Message::Clear),
            );

        Container::new(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .into()
    }
}
