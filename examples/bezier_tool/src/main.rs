//! This example showcases an interactive `Canvas` for drawing Bézier curves.
use iced::{
    button, Align, Button, Column, Element, Length, Sandbox, Settings, Text,
};

pub fn main() {
    Example::run(Settings {
        antialiasing: true,
        ..Settings::default()
    });
}

#[derive(Default)]
struct Example {
    bezier: bezier::State,
    curves: Vec<bezier::Curve>,
    button_state: button::State,
}

#[derive(Debug, Clone, Copy)]
enum Message {
    AddCurve(bezier::Curve),
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
            Message::AddCurve(curve) => {
                self.curves.push(curve);
                self.bezier.request_redraw();
            }
            Message::Clear => {
                self.bezier = bezier::State::default();
                self.curves.clear();
            }
        }
    }

    fn view(&mut self) -> Element<Message> {
        Column::new()
            .padding(20)
            .spacing(20)
            .align_items(Align::Center)
            .push(
                Text::new("Bezier tool example")
                    .width(Length::Shrink)
                    .size(50),
            )
            .push(self.bezier.view(&self.curves).map(Message::AddCurve))
            .push(
                Button::new(&mut self.button_state, Text::new("Clear"))
                    .padding(8)
                    .on_press(Message::Clear),
            )
            .into()
    }
}

mod bezier {
    use iced::{
        canvas::{self, Canvas, Event, Frame, Geometry, Path, Stroke},
        mouse, ButtonState, Element, Length, MouseCursor, Point, Rectangle,
        Size,
    };

    #[derive(Default)]
    pub struct State {
        pending: Option<Pending>,
        cursor_position: Point,
        cache: canvas::Cache,
    }

    impl State {
        pub fn view<'a>(
            &'a mut self,
            curves: &'a [Curve],
        ) -> Element<'a, Curve> {
            Canvas::new(Bezier {
                state: self,
                curves,
            })
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
        }

        pub fn request_redraw(&mut self) {
            self.cache.clear()
        }
    }

    struct Bezier<'a> {
        state: &'a mut State,
        curves: &'a [Curve],
    }

    impl<'a> canvas::Program<Curve> for Bezier<'a> {
        fn update(&mut self, event: Event, bounds: Size) -> Option<Curve> {
            match event {
                Event::Mouse(mouse_event) => match mouse_event {
                    mouse::Event::CursorMoved { x, y } => {
                        self.state.cursor_position = Point::new(x, y);

                        None
                    }
                    mouse::Event::Input {
                        button: mouse::Button::Left,
                        state: ButtonState::Pressed,
                    } if Rectangle::with_size(bounds)
                        .contains(self.state.cursor_position) =>
                    {
                        match self.state.pending {
                            None => {
                                self.state.pending = Some(Pending::One {
                                    from: self.state.cursor_position,
                                });
                                None
                            }
                            Some(Pending::One { from }) => {
                                self.state.pending = Some(Pending::Two {
                                    from,
                                    to: self.state.cursor_position,
                                });

                                None
                            }
                            Some(Pending::Two { from, to }) => {
                                self.state.pending = None;

                                Some(Curve {
                                    from,
                                    to,
                                    control: self.state.cursor_position,
                                })
                            }
                        }
                    }
                    _ => None,
                },
            }
        }

        fn draw(&self, bounds: Size) -> Vec<Geometry> {
            let content = self.state.cache.draw(bounds, |frame: &mut Frame| {
                Curve::draw_all(self.curves, frame);

                frame.stroke(
                    &Path::rectangle(Point::ORIGIN, frame.size()),
                    Stroke::default(),
                );
            });

            if let Some(pending) = &self.state.pending {
                let pending_curve =
                    pending.draw(bounds, self.state.cursor_position);

                vec![content, pending_curve]
            } else {
                vec![content]
            }
        }

        fn mouse_cursor(&self, bounds: Size) -> MouseCursor {
            if Rectangle::with_size(bounds).contains(self.state.cursor_position)
            {
                MouseCursor::Crosshair
            } else {
                MouseCursor::default()
            }
        }
    }

    #[derive(Debug, Clone, Copy)]
    pub struct Curve {
        from: Point,
        to: Point,
        control: Point,
    }

    impl Curve {
        fn draw_all(curves: &[Curve], frame: &mut Frame) {
            let curves = Path::new(|p| {
                for curve in curves {
                    p.move_to(curve.from);
                    p.quadratic_curve_to(curve.control, curve.to);
                }
            });

            frame.stroke(&curves, Stroke::default().with_width(2.0));
        }
    }

    #[derive(Debug, Clone, Copy)]
    enum Pending {
        One { from: Point },
        Two { from: Point, to: Point },
    }

    impl Pending {
        fn draw(&self, bounds: Size, cursor_position: Point) -> Geometry {
            let mut frame = Frame::new(bounds);

            match *self {
                Pending::One { from } => {
                    let line = Path::line(from, cursor_position);
                    frame.stroke(&line, Stroke::default().with_width(2.0));
                }
                Pending::Two { from, to } => {
                    let curve = Curve {
                        from,
                        to,
                        control: cursor_position,
                    };

                    Curve::draw_all(&[curve], &mut frame);
                }
            };

            frame.into_geometry()
        }
    }
}
