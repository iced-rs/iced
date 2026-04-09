use iced::widget::{
    center, checkbox, column, combo_box, container, directional, pick_list, progress_bar, radio,
    row, scrollable, slider, space, text, text_editor, text_input, toggler,
};
use iced::{Direction, Element, Length};

pub fn main() -> iced::Result {
    iced::application(App::default, App::update, App::view)
        .title("Directional Example")
        .run()
}

struct App {
    toggled: bool,
    checked: bool,
    language: Option<Language>,
    progress: f32,
    name: String,
    languages: combo_box::State<Language>,
    // A `Content` owns the editor buffer, including its alignment, so each
    // editor needs its own. Sharing one would let the panel laid out last
    // decide the alignment of both.
    notes: [text_editor::Content; 2],
}

impl Default for App {
    fn default() -> Self {
        Self {
            toggled: false,
            checked: false,
            language: None,
            progress: 0.0,
            name: String::new(),
            languages: combo_box::State::new(vec![Language::English, Language::Persian]),
            notes: [text_editor::Content::new(), text_editor::Content::new()],
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Language {
    English,
    Persian,
}

impl std::fmt::Display for Language {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Language::English => "English",
            Language::Persian => "فارسی",
        })
    }
}

#[derive(Debug, Clone)]
enum Message {
    Toggled(bool),
    Checked(bool),
    Selected(Language),
    LanguageSelected(Language),
    ProgressChanged(f32),
    NameChanged(String),
    NotesEdited(usize, text_editor::Action),
}

impl App {
    fn update(&mut self, message: Message) {
        match message {
            Message::Toggled(value) => self.toggled = value,
            Message::Checked(value) => self.checked = value,
            Message::Selected(choice) => self.language = Some(choice),
            Message::LanguageSelected(language) => self.language = Some(language),
            Message::ProgressChanged(value) => self.progress = value,
            Message::NameChanged(name) => self.name = name,
            Message::NotesEdited(panel, action) => self.notes[panel].perform(action),
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let content = |panel: usize| {
            column![
                row![
                    toggler(self.toggled)
                        .label("فارسی")
                        .on_toggle(Message::Toggled),
                    toggler(self.toggled)
                        .label("English")
                        .on_toggle(Message::Toggled),
                ]
                .spacing(20)
                .width(Length::Fill)
                .wrap(),
                checkbox(self.checked)
                    .label("English / فارسی")
                    .on_toggle(Message::Checked),
                row![
                    radio(
                        "English",
                        Language::English,
                        self.language,
                        Message::Selected
                    ),
                    radio("فارسی", Language::Persian, self.language, Message::Selected)
                ]
                .spacing(4)
                .wrap(),
                container(
                    column![
                        text("Container alignment demo"),
                        container(text("Default container alignment"))
                            .padding(6)
                            .width(Length::Fill)
                            .style(container::bordered_box),
                        container(text("Explicit align_left"))
                            .align_left(Length::Fill)
                            .padding(6)
                            .style(container::bordered_box),
                        container(text("Explicit align_right"))
                            .align_right(Length::Fill)
                            .padding(6)
                            .style(container::bordered_box),
                    ]
                    .spacing(8),
                )
                .padding(10)
                .width(Length::Fill)
                .style(container::bordered_box),
                pick_list(
                    self.language,
                    [Language::English, Language::Persian],
                    Language::to_string,
                )
                .placeholder("Choose language / انتخاب زبان")
                .on_select(Message::LanguageSelected)
                .width(Length::Fill),
                combo_box(
                    &self.languages,
                    "Search language / جستجوی زبان",
                    self.language.as_ref(),
                    Message::LanguageSelected,
                ),
                text_input("Your name / نام شما", &self.name).on_input(Message::NameChanged),
                text_editor(&self.notes[panel])
                    .placeholder("Notes / یادداشت‌ها")
                    .height(80)
                    .on_action(move |action| Message::NotesEdited(panel, action)),
                scrollable(column![
                    text("Scrollbar side demo"),
                    space().height(400),
                    text("Bottom of the scroll area"),
                ])
                .spacing(5)
                .height(100),
                progress_bar(0.0..=100.0, self.progress),
                slider(0.0..=100.0, self.progress, Message::ProgressChanged),
            ]
            .spacing(10)
        };

        let ltr = column![
            text("LTR"),
            container(directional(Direction::LeftToRight, content(0)))
                .padding(10)
                .width(Length::Fill)
                .style(container::bordered_box),
        ]
        .spacing(5);

        let rtl = column![
            text("RTL"),
            container(directional(Direction::RightToLeft, content(1)))
                .padding(10)
                .width(Length::Fill)
                .style(container::bordered_box),
        ]
        .spacing(5);

        center(scrollable(row![ltr, rtl].spacing(20))).into()
    }
}
