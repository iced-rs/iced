use iced::alignment::{self, Alignment};
use iced::widget::canvas::{Cursor, Frame, Text};
use iced::widget::{
    canvas, checkbox, column, horizontal_space, row, slider, text,
};
use iced::{
    Color, Element, Font, Length, Point, Rectangle, Sandbox, Settings, Theme,
    Vector,
};

pub fn main() -> iced::Result {
    VectorialText::run(Settings {
        antialiasing: true,
        ..Settings::default()
    })
}

struct VectorialText {
    state: State,
}

#[derive(Debug, Clone, Copy)]
enum Message {
    SizeChanged(f32),
    AngleChanged(f32),
    ScaleChanged(f32),
    ToggleJapanese(bool),
}

impl Sandbox for VectorialText {
    type Message = Message;

    fn new() -> Self {
        Self {
            state: State::new(),
        }
    }

    fn title(&self) -> String {
        String::from("Vectorial Text - Iced")
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::SizeChanged(size) => {
                self.state.size = size;
            }
            Message::AngleChanged(angle) => {
                self.state.angle = angle;
            }
            Message::ScaleChanged(scale) => {
                self.state.scale = scale;
            }
            Message::ToggleJapanese(use_japanese) => {
                self.state.use_japanese = use_japanese;
            }
        }
    }

    fn view(&self) -> Element<Message> {
        let slider_with_label = |label, range, value, message: fn(f32) -> _| {
            column![
                row![
                    text(label),
                    horizontal_space(Length::Fill),
                    text(format!("{:.2}", value))
                ],
                slider(range, value, message).step(0.01)
            ]
            .width(Length::Fill)
            .spacing(2)
        };

        column![
            canvas(&self.state).width(Length::Fill).height(Length::Fill),
            column![
                checkbox(
                    "Use Japanese",
                    self.state.use_japanese,
                    Message::ToggleJapanese
                ),
                row![
                    slider_with_label(
                        "Size",
                        2.0..=80.0,
                        self.state.size,
                        Message::SizeChanged,
                    ),
                    slider_with_label(
                        "Angle",
                        0.0..=360.0,
                        self.state.angle,
                        Message::AngleChanged,
                    ),
                    slider_with_label(
                        "Scale",
                        1.0..=20.0,
                        self.state.scale,
                        Message::ScaleChanged,
                    ),
                ]
                .spacing(20),
            ]
            .align_items(Alignment::Center)
            .spacing(10)
        ]
        .spacing(10)
        .padding(20)
        .into()
    }

    fn theme(&self) -> Theme {
        Theme::Dark
    }
}

struct State {
    size: f32,
    angle: f32,
    scale: f32,
    use_japanese: bool,
}

impl State {
    pub fn new() -> Self {
        Self {
            size: 40.0,
            angle: 0.0,
            scale: 1.0,
            use_japanese: false,
        }
    }
}

impl<Message> canvas::Program<Message> for State {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        _theme: &Theme,
        text_cache: &canvas::text::Cache,
        bounds: Rectangle,
        _cursor: Cursor,
    ) -> Vec<canvas::Geometry> {
        let mut frame = Frame::new(bounds.size());
        let center = bounds.center();

        frame.translate(Vector::new(center.x, center.y));
        frame.scale(self.scale);
        frame.rotate(self.angle * std::f32::consts::PI / 180.0);

        frame.fill_text(
            text_cache,
            Text {
                position: Point::new(0.0, self.size),
                color: Color::WHITE,
                font: Font::Default,
                size: self.size,
                content: String::from(if self.use_japanese {
                    "ベクトルテキスト🎉"
                } else {
                    "Vectorial Text! 🎉"
                }),
                horizontal_alignment: alignment::Horizontal::Center,
                vertical_alignment: alignment::Vertical::Center,
            },
        );

        vec![frame.into_geometry()]
    }
}
