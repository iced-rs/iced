use iced::{
    button, combo_box, Button, Column, ComboBox, Container, Element, Length,
    Sandbox, Settings, Text,
};

pub fn main() {
    Example::run(Settings::default())
}

#[derive(Default)]
struct Example {
    button: button::State,
    combo_box: combo_box::State,
    selected_language: Language,
}

#[derive(Debug, Clone, Copy)]
enum Message {
    ButtonPressed,
    LanguageSelected(Language),
}

impl Sandbox for Example {
    type Message = Message;

    fn new() -> Self {
        Self::default()
    }

    fn title(&self) -> String {
        String::from("Combo box - Iced")
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::ButtonPressed => {}
            Message::LanguageSelected(language) => {
                self.selected_language = language;
            }
        }
    }

    fn view(&mut self) -> Element<Message> {
        let combo_box = ComboBox::new(
            &mut self.combo_box,
            &Language::ALL[..],
            Some(self.selected_language),
            Message::LanguageSelected,
        );

        let button = Button::new(&mut self.button, Text::new("Press me!"))
            .on_press(Message::ButtonPressed);

        let mut content = Column::new()
            .spacing(10)
            .push(Text::new("Which is your favorite language?"))
            .push(combo_box);

        if self.selected_language == Language::Javascript {
            content = content.push(Text::new("You are wrong!"));
        }

        content = content.push(button);

        Container::new(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .into()
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
