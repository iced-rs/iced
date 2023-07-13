use iced::widget::{
    column, combo_box, container, scrollable, text, vertical_space,
};
use iced::{Alignment, Element, Length, Sandbox, Settings};

pub fn main() -> iced::Result {
    Example::run(Settings::default())
}

struct Example {
    languages: combo_box::State<Language>,
    selected_language: Option<Language>,
    text: String,
}

#[derive(Debug, Clone, Copy)]
enum Message {
    LanguageSelected(Language),
    LanguagePreview(Language),
    LanguageBlurred,
}

impl Sandbox for Example {
    type Message = Message;

    fn new() -> Self {
        Self {
            languages: combo_box::State::new(Language::ALL.to_vec()),
            selected_language: None,
            text: String::new(),
        }
    }

    fn title(&self) -> String {
        String::from("Combo box - Iced")
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::LanguageSelected(language) => {
                self.selected_language = Some(language);
                self.text = language.hello().to_string();
                self.languages.unfocus();
            }
            Message::LanguagePreview(language) => {
                self.text = language.hello().to_string();
            }
            Message::LanguageBlurred => {
                self.text = self
                    .selected_language
                    .map(|language| language.hello().to_string())
                    .unwrap_or_default();
            }
        }
    }

    fn view(&self) -> Element<Message> {
        let combo_box = combo_box(
            &self.languages,
            "Type a language...",
            self.selected_language.as_ref(),
            Message::LanguageSelected,
        )
        .on_selection(Message::LanguagePreview)
        .on_blur(Message::LanguageBlurred)
        .width(250);

        let content = column![
            "What is your language?",
            combo_box,
            vertical_space(150),
            text(&self.text),
        ]
        .width(Length::Fill)
        .align_items(Alignment::Center)
        .spacing(10);

        container(scrollable(content))
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .into()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Language {
    Danish,
    #[default]
    English,
    French,
    German,
    Italian,
    Portuguese,
    Spanish,
    Other,
}

impl Language {
    const ALL: [Language; 8] = [
        Language::Danish,
        Language::English,
        Language::French,
        Language::German,
        Language::Italian,
        Language::Portuguese,
        Language::Spanish,
        Language::Other,
    ];

    fn hello(&self) -> &str {
        match self {
            Language::Danish => "Halloy!",
            Language::English => "Hello!",
            Language::French => "Salut!",
            Language::German => "Hallo!",
            Language::Italian => "Ciao!",
            Language::Portuguese => "Olá!",
            Language::Spanish => "¡Hola!",
            Language::Other => "... hello?",
        }
    }
}

impl std::fmt::Display for Language {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Language::Danish => "Danish",
                Language::English => "English",
                Language::French => "French",
                Language::German => "German",
                Language::Italian => "Italian",
                Language::Portuguese => "Portuguese",
                Language::Spanish => "Spanish",
                Language::Other => "Some other language",
            }
        )
    }
}
