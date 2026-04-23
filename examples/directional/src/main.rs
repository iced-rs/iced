use iced::widget::{
    center, checkbox, column, container, directional, pick_list, progress_bar, radio, row, slider,
    text, toggler,
};
use iced::{Direction, Element, Length};

pub fn main() -> iced::Result {
    iced::application(App::default, App::update, App::view)
        .title("Directional Example")
        .run()
}

#[derive(Default)]
struct App {
    toggled: bool,
    checked: bool,
    language: Option<Language>,
    progress: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Language {
    English,
    Persian,
}

#[derive(Debug, Clone, Copy)]
enum Message {
    Toggled(bool),
    Checked(bool),
    Selected(Language),
    LanguageSelected(Language),
    ProgressChanged(f32),
}

impl App {
    fn update(&mut self, message: Message) {
        match message {
            Message::Toggled(value) => self.toggled = value,
            Message::Checked(value) => self.checked = value,
            Message::Selected(choice) => self.language = Some(choice),
            Message::LanguageSelected(language) => self.language = Some(language),
            Message::ProgressChanged(value) => self.progress = value,
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let content = || {
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
                    |language| match language {
                        Language::English => "English".to_owned(),
                        Language::Persian => "فارسی".to_owned(),
                    },
                )
                .placeholder("Choose language / انتخاب زبان")
                .on_select(Message::LanguageSelected)
                .width(Length::Fill),
                progress_bar(0.0..=100.0, self.progress),
                slider(0.0..=100.0, self.progress, Message::ProgressChanged),
            ]
            .spacing(10)
        };

        let ltr = column![
            text("LTR"),
            container(directional(Direction::LeftToRight, content()))
                .padding(10)
                .width(Length::Fill)
                .style(container::bordered_box),
        ]
        .spacing(5);

        let rtl = column![
            text("RTL"),
            container(directional(Direction::RightToLeft, content()))
                .padding(10)
                .width(Length::Fill)
                .style(container::bordered_box),
        ]
        .spacing(5);

        center(row![ltr, rtl].spacing(20)).into()
    }
}
