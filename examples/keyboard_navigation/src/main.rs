use iced::widget::{
    button, checkbox, column, container, pick_list, radio, slider, text,
    text_input, toggler, Row,
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
                text_input_value: "Hello world!".to_string(),
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
                self.theme = match theme {
                    ThemeType::Light => Theme::Light,
                    ThemeType::Dark => Theme::Dark,
                };

                Command::none()
            }
            Message::TextChanged(_) => Command::none(),
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
        let choose_theme = [ThemeType::Light, ThemeType::Dark].iter().fold(
            column![text("Choose a theme:")].spacing(10),
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
        );

        let pick_list = pick_list(
            &Language::ALL[..],
            self.selected_language,
            Message::LanguageSelected,
        )
        .placeholder("Choose a language...");

        let row_1 = Row::with_children(vec![choose_theme.into()])
            .spacing(20)
            .width(Length::Fill)
            .height(Length::Fill);

        let row_2 = Row::with_children(vec![
            button("*").on_press(Message::ButtonPressed).into(),
            checkbox("*", self.checkbox_value, Message::CheckboxToggled).into(),
            text_input("*", &self.text_input_value, Message::TextChanged)
                .into(),
            toggler(
                "*".to_owned(),
                self.toggler_value,
                Message::TogglerToggled,
            )
            .into(),
            slider(0.0..=1.0, self.slider_value, Message::SliderChanged)
                .step(0.25)
                .into(),
        ])
        .spacing(20)
        .width(Length::Fill)
        .height(Length::Shrink);

        let content = column![row_1, pick_list, row_2].spacing(20).padding(20);

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .into()
    }

    fn theme(&self) -> Theme {
        self.theme.clone()
    }
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
