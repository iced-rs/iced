use iced::theme::Palette;
use iced::widget::{
    checkbox, column, container, horizontal_space, row, slider, text,
};
use iced::{gradient, window};
use iced::{
    Alignment, Background, Color, Element, Length, Radians, Sandbox, Settings,
};

pub fn main() -> iced::Result {
    Gradient::run(Settings {
        window: window::Settings {
            transparent: true,
            ..Default::default()
        },
        ..Default::default()
    })
}

#[derive(Debug, Clone, Copy)]
struct Gradient {
    start: Color,
    end: Color,
    angle: Radians,
    transparent_window: bool,
}

#[derive(Debug, Clone, Copy)]
enum Message {
    StartChanged(Color),
    EndChanged(Color),
    AngleChanged(Radians),
    SetTransparent(bool),
}

impl Sandbox for Gradient {
    type Message = Message;

    fn new() -> Self {
        Self {
            start: Color::WHITE,
            end: Color::new(0.0, 0.0, 1.0, 1.0),
            angle: Radians(0.0),
            transparent_window: false,
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
            Message::SetTransparent(transparent) => {
                self.transparent_window = transparent;
            }
        }
    }

    fn view(&self) -> Element<Message> {
        let Self {
            start,
            end,
            angle,
            transparent_window,
        } = *self;

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

        let transparency_toggle = iced::widget::Container::new(
            checkbox("Transparent window", transparent_window)
                .on_toggle(Message::SetTransparent),
        )
        .padding(8);

        column![
            color_picker("Start", self.start).map(Message::StartChanged),
            color_picker("End", self.end).map(Message::EndChanged),
            angle_picker,
            transparency_toggle,
            gradient_box,
        ]
        .into()
    }

    fn theme(&self) -> iced::Theme {
        if self.transparent_window {
            iced::Theme::custom(
                String::new(),
                Palette {
                    background: Color::TRANSPARENT,
                    ..iced::Theme::default().palette()
                },
            )
        } else {
            iced::Theme::default()
        }
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
        slider(0.0..=1.0, color.a, move |a| { Color { a, ..color } })
            .step(0.01),
    ]
    .spacing(8)
    .padding(8)
    .align_items(Alignment::Center)
    .into()
}
