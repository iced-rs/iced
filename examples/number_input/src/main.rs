use iced::{
    number_input, window, Align, Container, Element, Length, NumberInput, Row,
    Sandbox, Settings, Text,
};

#[derive(Default)]
pub struct NumberInputDemo {
    state: number_input::State,
    value: u8,
}

#[derive(Debug, Clone)]
pub enum Message {
    NumInpChanged(u8),
}

fn main() -> iced::Result {
    NumberInputDemo::run(Settings {
        default_text_size: 14,
        window: window::Settings {
            size: (250, 200),
            ..Default::default()
        },
        ..Settings::default()
    })
}

impl Sandbox for NumberInputDemo {
    type Message = Message;

    fn new() -> Self {
        Self {
            value: 27,
            ..Self::default()
        }
    }

    fn title(&self) -> String {
        String::from("Number Input Demo")
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::NumInpChanged(val) => {
                self.value = val;
            }
        }
    }

    fn view(&mut self) -> Element<Message> {
        let lb_minute = Text::new("Number Input:");
        let txt_minute = NumberInput::new(
            &mut self.state,
            self.value,
            255,
            Message::NumInpChanged,
        )
        .step(1)
        .input_style(style::CustomTextInput)
        .style(style::CustomNumInput);

        Container::new(
            Row::new()
                .spacing(10)
                .align_items(Align::Center)
                .push(lb_minute)
                .push(txt_minute),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x()
        .center_y()
        .into()
    }
}

mod style {
    use iced::Color;
    use iced::{number_input, text_input};

    const BACKGROUND: Color =
        Color::from_rgb(238.0 / 255.0, 238.0 / 255.0, 238.0 / 255.0);
    const FOREGROUND: Color =
        Color::from_rgb(224.0 / 255.0, 224.0 / 255.0, 224.0 / 255.0);
    const HOVERED: Color =
        Color::from_rgb(129.0 / 255.0, 129.0 / 255.0, 129.0 / 255.0);
    const PRIMARY: Color =
        Color::from_rgb(12.0 / 255.0, 46.0 / 251.0, 179.0 / 255.0);

    pub struct CustomNumInput;
    impl number_input::StyleSheet for CustomNumInput {
        fn active(&self) -> number_input::Style {
            number_input::Style {
                icon_color: PRIMARY,
                ..number_input::Style::default()
            }
        }
    }

    pub struct CustomTextInput;
    impl text_input::StyleSheet for CustomTextInput {
        fn active(&self) -> text_input::Style {
            text_input::Style {
                background: BACKGROUND.into(),
                border_color: PRIMARY,
                border_width: 1.0,
                border_radius: 5.5,
                ..text_input::Style::default()
            }
        }

        fn focused(&self) -> text_input::Style {
            let active = self.active();

            text_input::Style {
                background: FOREGROUND.into(),
                ..active
            }
        }

        fn placeholder_color(&self) -> Color {
            HOVERED
        }

        fn selection_color(&self) -> Color {
            HOVERED
        }

        fn value_color(&self) -> Color {
            Color::BLACK
        }
    }
}
