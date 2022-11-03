use iced::widget::{
    button, column, horizontal_rule, horizontal_space, row, scrollable, text,
    text_input,
};
use iced::{Element, Sandbox};
use iced::{Length, Settings};
use iced_lazy::Cached;

use std::collections::HashSet;

pub fn main() -> iced::Result {
    App::run(Settings::default())
}

struct App {
    options: HashSet<String>,
    input: String,
    sort_order: SortOrder,
}

impl Default for App {
    fn default() -> Self {
        Self {
            options: ["Foo", "Bar", "Baz", "Qux", "Corge", "Waldo", "Fred"]
                .into_iter()
                .map(ToString::to_string)
                .collect(),
            input: Default::default(),
            sort_order: SortOrder::Ascending,
        }
    }
}

#[derive(Debug, Clone)]
enum Message {
    InputChanged(String),
    ToggleSortOrder,
    DeleteOption(String),
    AddOption(String),
}

impl Sandbox for App {
    type Message = Message;

    fn new() -> Self {
        Self::default()
    }

    fn title(&self) -> String {
        String::from("Cached - Iced")
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::InputChanged(input) => {
                self.input = input;
            }
            Message::ToggleSortOrder => {
                self.sort_order = match self.sort_order {
                    SortOrder::Ascending => SortOrder::Descending,
                    SortOrder::Descending => SortOrder::Ascending,
                }
            }
            Message::AddOption(option) => {
                self.options.insert(option);
                self.input.clear();
            }
            Message::DeleteOption(option) => {
                self.options.remove(&option);
            }
        }
    }

    fn view(&self) -> Element<Message> {
        let options =
            Cached::new((&self.sort_order, self.options.len()), || {
                let mut options = self.options.iter().collect::<Vec<_>>();
                options.sort_by(|a, b| match self.sort_order {
                    SortOrder::Ascending => {
                        a.to_lowercase().cmp(&b.to_lowercase())
                    }
                    SortOrder::Descending => {
                        b.to_lowercase().cmp(&a.to_lowercase())
                    }
                });

                options.into_iter().fold(
                    column![horizontal_rule(1)],
                    |column, option| {
                        column
                            .push(row![
                                text(option),
                                horizontal_space(Length::Fill),
                                button("Delete").on_press(
                                    Message::DeleteOption(option.to_string(),),
                                )
                            ])
                            .push(horizontal_rule(1))
                    },
                )
            });

        scrollable(
            column![
                button(text(format!(
                    "Toggle Sort Order ({})",
                    self.sort_order
                )))
                .on_press(Message::ToggleSortOrder),
                options,
                text_input(
                    "Add a new option",
                    &self.input,
                    Message::InputChanged,
                )
                .on_submit(Message::AddOption(self.input.clone())),
            ]
            .spacing(20)
            .padding(20),
        )
        .into()
    }
}

#[derive(Debug, Hash)]
enum SortOrder {
    Ascending,
    Descending,
}

impl std::fmt::Display for SortOrder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Ascending => "Ascending",
                Self::Descending => "Descending",
            }
        )
    }
}
