use iced::widget::{
    button, container, pick_list, radio, slider, text, text_input,
    toggler, Column, checkbox,
};
use iced::{executor, keyboard, subscription, Subscription};
use iced::{Application, Command, Element, Length, Settings, Theme};

pub fn main() -> iced::Result {
    KeyboardNavigationDemo::run(Settings::default())
}

struct KeyboardNavigationDemo {
    theme: Theme,
    text_input_value: String,
    checkbox_value: bool,
    toggler_value: bool,
    slider_value: f32,
    selected_language: Option<Language>,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum ThemeType {
    Light,
    Dark,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Language {
    Rust,
    Elm,
    Ruby,
    Haskell,
    C,
    Javascript,
    Other,
}

impl Language {
    const ALL: [Language; 7] = [
        Language::C,
        Language::Elm,
        Language::Ruby,
        Language::Haskell,
        Language::Rust,
        Language::Javascript,
        Language::Other,
    ];
}

impl Default for Language {
    fn default() -> Language {
        Language::Rust
    }
}

impl std::fmt::Display for Language {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Language::Rust => "Rust",
                Language::Elm => "Elm",
                Language::Ruby => "Ruby",
                Language::Haskell => "Haskell",
                Language::C => "C",
                Language::Javascript => "Javascript",
                Language::Other => "Some other language",
            }
        )
    }
}

#[derive(Debug, Clone)]
enum Message {
    ThemeChanged(ThemeType),
    TextChanged(String),
    TabPressed,
    ButtonPressed,
    CheckboxToggled(bool),
    TogglerToggled(bool),
    SliderChanged(f32),
    LanguageSelected(Language),
}

impl Application for KeyboardNavigationDemo {
    type Message = Message;
    type Theme = Theme;
    type Executor = executor::Default;
    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, Command<Message>) {
        (
            KeyboardNavigationDemo {
                theme: Default::default(),
                text_input_value: "Text Input".to_string(),
                checkbox_value: false,
                toggler_value: false,
                slider_value: 0.5,
                selected_language: Language::ALL.first().cloned(),
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        String::from("Keyboard Navigation - Iced")
    }

    fn subscription(&self) -> Subscription<Message> {
        subscription::events_with(|event, status| match (event, status) {
            (
                iced::Event::Keyboard(keyboard::Event::KeyPressed {
                    key_code: keyboard::KeyCode::Tab,
                    modifiers: _,
                    ..
                }),
                iced::event::Status::Ignored,
            ) => Some(Message::TabPressed),
            _ => None,
        })
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::ThemeChanged(theme) => {
                println!("Radio changed!");
                self.theme = match theme {
                    ThemeType::Light => Theme::Light,
                    ThemeType::Dark => Theme::Dark,
                };

                Command::none()
            }
            Message::TextChanged(value) => {
                println!("Text changed!");
                self.text_input_value = value;
                Command::none()
            },
            Message::ButtonPressed => {
                println!("Button pressed!");
                Command::none()
            }
            Message::CheckboxToggled(checked) => {
                println!("Box toggled! {}", checked);
                self.checkbox_value = checked;
                Command::none()
            }
            Message::TogglerToggled(value) => {
                println!("Toggler toggled! {}", value);
                self.toggler_value = value;
                Command::none()
            }
            Message::SliderChanged(value) => {
                println!("Slider changed! {}", value);
                self.slider_value = value;
                Command::none()
            }
            Message::LanguageSelected(language) => {
                println!("Language selected! {:?}", language);
                self.selected_language = Some(language);
                Command::none()
            }
            Message::TabPressed => iced::widget::focus_next(),
        }
    }

    fn view(&self) -> Element<Message> {
        let radio_example = container(
            Column::with_children(vec![[ThemeType::Light, ThemeType::Dark]
                .iter()
                .fold(
                    Column::with_children(vec![text("Radio").into()])
                        .spacing(10),
                    |column, theme| {
                        column.push(radio(
                            format!("{:?}", theme),
                            *theme,
                            Some(match self.theme {
                                Theme::Light => ThemeType::Light,
                                Theme::Dark => ThemeType::Dark,
                                Theme::Custom(_) => ThemeType::Dark,
                            }),
                            Message::ThemeChanged,
                        ))
                    },
                )
                .into()])
            .width(Length::Fill)
            .height(Length::Fill),
        )
        .into();

        let list_picker_example = container(
            Column::with_children(vec![
                text("List Picker").into(),
                pick_list(
                    &Language::ALL[..],
                    self.selected_language,
                    Message::LanguageSelected,
                )
                .placeholder("Choose a language...")
                .into(),
            ])
            .width(Length::Fill)
            .height(Length::Fill),
        )
        .into();

        let button_example = container(
            Column::with_children(vec![button("Button")
                .on_press(Message::ButtonPressed)
                .into()])
            .width(Length::Fill)
            .height(Length::Fill),
        )
        .into();

        let checkbox_example = container(
            Column::with_children(vec![
                checkbox("*", self.checkbox_value, Message::CheckboxToggled).into(),
            ])
            .width(Length::Fill)
            .height(Length::Fill),
        )
        .into();

        let text_input_example = container(
            Column::with_children(vec![text_input(
                "Text Input",
                &self.text_input_value,
                Message::TextChanged,
            )
            .into()])
            .width(Length::Fill)
            .height(Length::Fill),
        )
        .into();

        let toggler_example = container(
            Column::with_children(vec![
                text("Toggler").into(),
                toggler(
                    "Toggle".to_owned(),
                    self.toggler_value,
                    Message::TogglerToggled,
                )
                .width(Length::Units(35))
                .into(),
            ])
            .width(Length::Fill)
            .height(Length::Fill),
        )
        .into();

        let slider_example = container(
            Column::with_children(vec![
                text("Slider").into(),
                text("Enter or Space to toggle grab").into(),
                slider(0.0..=1.0, self.slider_value, Message::SliderChanged)
                    .step(0.25)
                    .into()
            ])
            .width(Length::Fill)
            .height(Length::Fill),
        )
        .into();



        let content = Column::with_children(vec![
            radio_example,
            list_picker_example,
            button_example,
            checkbox_example,
            text_input_example,
            toggler_example,
            slider_example
        ])
        .spacing(20)
        .padding(20)
        .width(Length::Units(250));

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    fn theme(&self) -> Theme {
        self.theme.clone()
    }
}
