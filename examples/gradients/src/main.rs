use iced::widget::{column, container, row, slider, text};
use iced::{
    gradient, Alignment, Background, BorderRadius, Color, Element, Length,
    Radians, Sandbox, Settings,
};

pub fn main() -> iced::Result {
    Gradient::run(Settings::default())
}

struct Gradient {
    first: Color,
    second: Color,
    angle: f32,
}

#[derive(Debug, Clone, Copy)]
enum Message {
    FirstChanged(Color),
    SecondChanged(Color),
    AngleChanged(f32),
}

impl Sandbox for Gradient {
    type Message = Message;

    fn new() -> Self {
        let first = Color::new(0.2784314, 0.0627451, 0.4117647, 1.0);
        let second = Color::new(0.1882353, 0.772549, 0.8235294, 1.0);

        Self {
            first,
            second,
            angle: 0.0,
        }
    }

    fn title(&self) -> String {
        String::from("Color gradient")
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::FirstChanged(color) => self.first = color,
            Message::SecondChanged(color) => self.second = color,
            Message::AngleChanged(angle) => self.angle = angle,
        }
    }

    fn view(&self) -> Element<Message> {
        let first = self.first;
        let second = self.second;
        let angle = self.angle;

        let gradient_box = container(text(""))
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .style(move |_: &_| {
                let gradient = gradient::Linear::new(Radians(angle))
                    .add_stop(0.0, first)
                    .add_stop(1.0, second)
                    .into();

                container::Appearance {
                    text_color: None,
                    background: Some(Background::Gradient(gradient)),
                    border_radius: BorderRadius::default(),
                    border_width: 0.0,
                    border_color: Color::new(0.0, 0.0, 0.0, 0.0),
                }
            });

        let range = 0.0..=1.0;
        let l = self.first;
        let r = self.second;

        let first_color_picker = row![
            text("First").width(64),
            slider(range.clone(), l.r, move |v| {
                Message::FirstChanged(Color::new(v, l.g, l.b, l.a))
            })
            .step(0.01),
            slider(range.clone(), l.g, move |v| {
                Message::FirstChanged(Color::new(l.r, v, l.b, l.a))
            })
            .step(0.01),
            slider(range.clone(), l.b, move |v| {
                Message::FirstChanged(Color::new(l.r, l.g, v, l.a))
            })
            .step(0.01),
        ]
        .spacing(8)
        .padding(8)
        .align_items(Alignment::Center);

        let second_color_picker = row![
            text("Second").width(64),
            slider(range.clone(), r.r, move |v| {
                Message::SecondChanged(Color::new(v, r.g, r.b, r.a))
            })
            .step(0.01),
            slider(range.clone(), r.g, move |v| {
                Message::SecondChanged(Color::new(r.r, v, r.b, r.a))
            })
            .step(0.01),
            slider(range.clone(), r.b, move |v| {
                Message::SecondChanged(Color::new(r.r, r.g, v, r.a))
            })
            .step(0.01),
        ]
        .spacing(8)
        .padding(8)
        .align_items(Alignment::Center);

        let angle_picker = row![
            text("Angle").width(64),
            slider(0.0..=std::f32::consts::PI * 2.0, self.angle, move |v| {
                Message::AngleChanged(v)
            })
            .step(0.01)
        ]
        .spacing(8)
        .padding(8)
        .align_items(Alignment::Center);

        column![
            first_color_picker,
            second_color_picker,
            angle_picker,
            gradient_box
        ]
        .into()
    }
}
