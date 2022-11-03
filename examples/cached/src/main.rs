use iced::theme;
use iced::widget::{
    button, column, horizontal_space, row, scrollable, text, text_input,
};
use iced::{Element, Length, Sandbox, Settings};
use iced_lazy::lazy;

use std::collections::HashSet;

pub fn main() -> iced::Result {
    App::run(Settings::default())
}

struct App {
    options: HashSet<String>,
    input: String,
    order: Order,
}

impl Default for App {
    fn default() -> Self {
        Self {
            options: ["Foo", "Bar", "Baz", "Qux", "Corge", "Waldo", "Fred"]
                .into_iter()
                .map(ToString::to_string)
                .collect(),
            input: Default::default(),
            order: Order::Ascending,
        }
    }
}

#[derive(Debug, Clone)]
enum Message {
    InputChanged(String),
    ToggleOrder,
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
            Message::ToggleOrder => {
                self.order = match self.order {
                    Order::Ascending => Order::Descending,
                    Order::Descending => Order::Ascending,
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
        let options = lazy((&self.order, self.options.len()), || {
            let mut options: Vec<_> = self.options.iter().collect();

            options.sort_by(|a, b| match self.order {
                Order::Ascending => a.to_lowercase().cmp(&b.to_lowercase()),
                Order::Descending => b.to_lowercase().cmp(&a.to_lowercase()),
            });

            column(
                options
                    .into_iter()
                    .map(|option| {
                        row![
                            text(option),
                            horizontal_space(Length::Fill),
                            button("Delete")
                                .on_press(Message::DeleteOption(
                                    option.to_string(),
                                ),)
                                .style(theme::Button::Destructive)
                        ]
                        .into()
                    })
                    .collect(),
            )
            .spacing(10)
        });

        column![
            scrollable(options).height(Length::Fill),
            row![
                text_input(
                    "Add a new option",
                    &self.input,
                    Message::InputChanged,
                )
                .on_submit(Message::AddOption(self.input.clone())),
                button(text(format!("Toggle Order ({})", self.order)))
                    .on_press(Message::ToggleOrder)
            ]
            .spacing(10)
        ]
        .spacing(20)
        .padding(20)
        .into()
    }
}

#[derive(Debug, Hash)]
enum Order {
    Ascending,
    Descending,
}

impl std::fmt::Display for Order {
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
