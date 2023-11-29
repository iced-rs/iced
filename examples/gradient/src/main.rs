use iced::gradient;
use iced::widget::{column, container, horizontal_space, row, slider, text};
use iced::{
    Alignment, Background, Color, Element, Length, Radians, Sandbox, Settings,
};

pub fn main() -> iced::Result {
    Gradient::run(Settings::default())
}

#[derive(Debug, Clone, Copy)]
struct Gradient {
    start: Color,
    end: Color,
    angle: Radians,
}

#[derive(Debug, Clone, Copy)]
enum Message {
    StartChanged(Color),
    EndChanged(Color),
    AngleChanged(Radians),
}

impl Sandbox for Gradient {
    type Message = Message;

    fn new() -> Self {
        Self {
            start: Color::WHITE,
            end: Color::new(0.0, 0.0, 1.0, 1.0),
            angle: Radians(0.0),
        }
    }

    fn title(&self) -> String {
        String::from("Gradient")
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::StartChanged(color) => self.start = color,
            Message::EndChanged(color) => self.end = color,
            Message::AngleChanged(angle) => self.angle = angle,
        }
    }

    fn view(&self) -> Element<Message> {
        let Self { start, end, angle } = *self;

        let gradient_box = container(horizontal_space(Length::Fill))
            .width(Length::Fill)
            .height(Length::Fill)
            .style(move |_: &_| {
                let gradient = gradient::Linear::new(angle)
                    .add_stop(0.0, start)
                    .add_stop(1.0, end)
                    .into();

                container::Appearance {
                    background: Some(Background::Gradient(gradient)),
                    ..Default::default()
                }
            });

        let angle_picker = row![
            text("Angle").width(64),
            slider(Radians::RANGE, self.angle, Message::AngleChanged)
                .step(0.01)
        ]
        .spacing(8)
        .padding(8)
        .align_items(Alignment::Center);

        column![
            color_picker("Start", self.start).map(Message::StartChanged),
            color_picker("End", self.end).map(Message::EndChanged),
            angle_picker,
            gradient_box
        ]
        .into()
    }
}

fn color_picker(label: &str, color: Color) -> Element<'_, Color> {
    row![
        text(label).width(64),
        slider(0.0..=1.0, color.r, move |r| { Color { r, ..color } })
            .step(0.01),
        slider(0.0..=1.0, color.g, move |g| { Color { g, ..color } })
            .step(0.01),
        slider(0.0..=1.0, color.b, move |b| { Color { b, ..color } })
            .step(0.01),
    ]
    .spacing(8)
    .padding(8)
    .align_items(Alignment::Center)
    .into()
}
