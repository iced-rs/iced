//! This example showcases a simple native MS paint like application.
mod paint {
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
    use lyon::lyon_tessellation::{
        basic_shapes, BuffersBuilder, StrokeAttributes, StrokeOptions,
        StrokeTessellator, VertexBuffers,
    };
    use std::sync::Arc;

    pub struct Paint<'a, Message> {
        state: &'a mut State,
        strokes: &'a [(Point, Point)],
        on_stroke: Box<dyn Fn((Point, Point)) -> Message>,
    }

    impl<'a, Message> Paint<'a, Message> {
        pub fn new<F>(
            strokes: &'a [(Point, Point)],
            state: &'a mut State,
            on_stroke: F,
        ) -> Self
        where
            F: 'static + Fn((Point, Point)) -> Message,
        {
            Self {
                state,
                strokes,
                on_stroke: Box::new(on_stroke),
            }
        }
    }

    #[derive(Debug, Clone, Copy, Default)]
    pub struct State {
        is_dragging: bool,
        previous_mouse_pos: Option<Point>,
    }

    impl<'a, Message> Widget<Message, Renderer> for Paint<'a, Message> {
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
            _cursor_position: Point,
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

            for (from, to) in self.strokes {
                path_builder.move_to(lyon::math::Point::new(
                    from.x + bounds.x,
                    from.y + bounds.y,
                ));
                path_builder.line_to(lyon::math::Point::new(
                    to.x + bounds.x,
                    to.y + bounds.y,
                ));
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
            _cursor_position: Point,
            messages: &mut Vec<Message>,
            _renderer: &Renderer,
            _clipboard: Option<&dyn Clipboard>,
        ) {
            let bounds = layout.bounds();
            match event {
                Event::Mouse(input::mouse::Event::CursorMoved { x, y })
                    if bounds.contains(Point::new(x, y)) =>
                {
                    let pos = Point::new(x - bounds.x, y - bounds.y);
                    if self.state.is_dragging {
                        if let Some(prev) = self.state.previous_mouse_pos {
                            messages.push((self.on_stroke)((prev, pos)));
                        }
                    }
                    self.state.previous_mouse_pos = Some(pos);
                }
                Event::Mouse(input::mouse::Event::Input { state, .. }) => {
                    match state {
                        input::ButtonState::Pressed => {
                            self.state.is_dragging = true;
                        }
                        input::ButtonState::Released => {
                            self.state.is_dragging = false;
                        }
                    }
                }
                _ => {}
            }
        }
    }

    impl<'a, Message> Into<Element<'a, Message, Renderer>> for Paint<'a, Message>
    where
        Message: 'static,
    {
        fn into(self) -> Element<'a, Message, Renderer> {
            Element::new(self)
        }
    }
}

use iced::{
    button, Align, Button, Color, Column, Container, Element, Length, Sandbox,
    Settings, Text,
};
use iced_native::Point;
use paint::Paint;

pub fn main() {
    Example::run(Settings::default())
}

#[derive(Default)]
struct Example {
    paint_state: paint::State,
    strokes: Vec<(Point, Point)>,
    button_state: button::State,
}

#[derive(Debug, Clone, Copy)]
enum Message {
    Stroke((Point, Point)),
    Clear,
}

impl Sandbox for Example {
    type Message = Message;

    fn new() -> Self {
        Example::default()
    }

    fn title(&self) -> String {
        String::from("Paint - Iced")
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::Stroke(stroke) => {
                self.strokes.push(stroke);
            }
            Message::Clear => {
                self.strokes.clear();
            }
        }
    }

    fn view(&mut self) -> Element<Message> {
        let content = Column::new()
            .padding(20)
            .spacing(20)
            .align_items(Align::Center)
            .push(Text::new("Paint example").width(Length::Shrink).size(50))
            .push(Paint::new(
                self.strokes.as_slice(),
                &mut self.paint_state,
                Message::Stroke,
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
