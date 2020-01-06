use iced::{
    button, text_input, Button, Column, Container, Element, Length, Radio, Row,
    Sandbox, Settings, Text, TextInput,
};

pub fn main() {
    Styling::run(Settings::default())
}

struct Styling {
    theme: style::Theme,
    input: text_input::State,
    input_value: String,
    button: button::State,
}

#[derive(Debug, Clone)]
enum Message {
    ThemeChanged(style::Theme),
    InputChanged(String),
    ButtonPressed,
}

impl Sandbox for Styling {
    type Message = Message;

    fn new() -> Self {
        Styling {
            theme: style::Theme::Light,
            input: text_input::State::default(),
            input_value: String::new(),
            button: button::State::default(),
        }
    }

    fn title(&self) -> String {
        String::from("Styling - Iced")
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::ThemeChanged(theme) => self.theme = theme,
            Message::InputChanged(value) => self.input_value = value,
            Message::ButtonPressed => (),
        }
    }

    fn view(&mut self) -> Element<Message> {
        let choose_theme = style::Theme::ALL.iter().fold(
            Column::new()
                .width(Length::Shrink)
                .spacing(10)
                .push(Text::new("Choose a theme:").width(Length::Shrink)),
            |column, theme| {
                column.push(Radio::new(
                    *theme,
                    &format!("{:?}", theme),
                    Some(self.theme),
                    Message::ThemeChanged,
                ))
            },
        );

        let text_input = TextInput::new(
            &mut self.input,
            "Type something...",
            &self.input_value,
            Message::InputChanged,
        )
        .padding(10)
        .size(20)
        .style(self.theme);

        let button = Button::new(&mut self.button, Text::new("Submit"))
            .padding(10)
            .on_press(Message::ButtonPressed)
            .style(self.theme);

        let content = Column::new()
            .spacing(20)
            .padding(20)
            .max_width(600)
            .push(choose_theme)
            .push(Row::new().spacing(10).push(text_input).push(button));

        Container::new(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .style(self.theme)
            .into()
    }
}

mod style {
    use iced::{button, container, text_input};

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum Theme {
        Light,
        Dark,
    }

    impl Theme {
        pub const ALL: [Theme; 2] = [Theme::Light, Theme::Dark];
    }

    impl From<Theme> for Box<dyn container::StyleSheet> {
        fn from(theme: Theme) -> Self {
            match theme {
                Theme::Light => light::Container.into(),
                Theme::Dark => dark::Container.into(),
            }
        }
    }

    impl From<Theme> for Box<dyn text_input::StyleSheet> {
        fn from(theme: Theme) -> Self {
            match theme {
                Theme::Light => light::TextInput.into(),
                Theme::Dark => dark::TextInput.into(),
            }
        }
    }

    impl From<Theme> for Box<dyn button::StyleSheet> {
        fn from(theme: Theme) -> Self {
            match theme {
                Theme::Light => light::Button.into(),
                Theme::Dark => dark::Button.into(),
            }
        }
    }

    mod light {
        use iced::{button, container, text_input, Background, Color, Vector};

        pub struct Container;

        impl container::StyleSheet for Container {
            fn style(&self) -> container::Style {
                container::Style::default()
            }
        }

        pub struct TextInput;

        impl text_input::StyleSheet for TextInput {
            fn active(&self) -> text_input::Style {
                text_input::Style {
                    background: Background::Color(Color::WHITE),
                    border_radius: 5,
                    border_width: 1,
                    border_color: Color::from_rgb(0.7, 0.7, 0.7),
                }
            }

            fn focused(&self) -> text_input::Style {
                text_input::Style {
                    border_width: 1,
                    border_color: Color::from_rgb(0.5, 0.5, 0.5),
                    ..self.active()
                }
            }

            fn placeholder_color(&self) -> Color {
                Color::from_rgb(0.7, 0.7, 0.7)
            }

            fn value_color(&self) -> Color {
                Color::from_rgb(0.3, 0.3, 0.3)
            }
        }

        pub struct Button;

        impl button::StyleSheet for Button {
            fn active(&self) -> button::Style {
                button::Style {
                    background: Some(Background::Color(Color::from_rgb(
                        0.11, 0.42, 0.87,
                    ))),
                    border_radius: 12,
                    shadow_offset: Vector::new(1.0, 1.0),
                    text_color: Color::from_rgb8(0xEE, 0xEE, 0xEE),
                    ..button::Style::default()
                }
            }

            fn hovered(&self) -> button::Style {
                button::Style {
                    text_color: Color::WHITE,
                    shadow_offset: Vector::new(1.0, 2.0),
                    ..self.active()
                }
            }
        }
    }

    mod dark {
        use iced::{button, container, text_input, Background, Color};

        pub struct Container;

        impl container::StyleSheet for Container {
            fn style(&self) -> container::Style {
                container::Style {
                    background: Some(Background::Color(Color::from_rgb8(
                        0x36, 0x39, 0x3F,
                    ))),
                    text_color: Some(Color::WHITE),
                    ..container::Style::default()
                }
            }
        }

        pub struct TextInput;

        impl text_input::StyleSheet for TextInput {
            fn active(&self) -> text_input::Style {
                text_input::Style {
                    background: Background::Color(Color::from_rgb8(
                        0x40, 0x44, 0x4B,
                    )),
                    border_radius: 2,
                    border_width: 0,
                    border_color: Color::TRANSPARENT,
                }
            }

            fn focused(&self) -> text_input::Style {
                text_input::Style {
                    border_width: 1,
                    border_color: Color::from_rgb8(0x6F, 0xFF, 0xE9),
                    ..self.active()
                }
            }

            fn hovered(&self) -> text_input::Style {
                text_input::Style {
                    border_width: 1,
                    border_color: Color::from_rgb8(0x5B, 0xC0, 0xBE),
                    ..self.focused()
                }
            }

            fn placeholder_color(&self) -> Color {
                Color::from_rgb(0.7, 0.7, 0.7)
            }

            fn value_color(&self) -> Color {
                Color::WHITE
            }
        }

        pub struct Button;

        impl button::StyleSheet for Button {
            fn active(&self) -> button::Style {
                button::Style {
                    background: Some(Background::Color(Color::from_rgb8(
                        0x72, 0x89, 0xDA,
                    ))),
                    border_radius: 3,
                    text_color: Color::WHITE,
                    ..button::Style::default()
                }
            }

            fn hovered(&self) -> button::Style {
                button::Style {
                    background: Some(Background::Color(Color::from_rgb8(
                        0x67, 0x7B, 0xC4,
                    ))),
                    text_color: Color::WHITE,
                    ..self.active()
                }
            }
        }
    }
}
