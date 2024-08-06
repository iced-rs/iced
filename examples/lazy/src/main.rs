use iced::widget::{
    button, column, horizontal_space, lazy, pick_list, row, scrollable, text,
    text_input,
};
use iced::{Element, Fill};

use std::collections::HashSet;
use std::hash::Hash;

pub fn main() -> iced::Result {
    iced::run("Lazy - Iced", App::update, App::view)
}

struct App {
    version: u8,
    items: HashSet<Item>,
    input: String,
    order: Order,
}

impl Default for App {
    fn default() -> Self {
        Self {
            version: 0,
            items: ["Foo", "Bar", "Baz", "Qux", "Corge", "Waldo", "Fred"]
                .into_iter()
                .map(From::from)
                .collect(),
            input: String::default(),
            order: Order::Ascending,
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
enum Color {
    #[default]
    Black,
    Red,
    Orange,
    Yellow,
    Green,
    Blue,
    Purple,
}

impl Color {
    const ALL: &'static [Color] = &[
        Color::Black,
        Color::Red,
        Color::Orange,
        Color::Yellow,
        Color::Green,
        Color::Blue,
        Color::Purple,
    ];
}

impl std::fmt::Display for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Black => "Black",
            Self::Red => "Red",
            Self::Orange => "Orange",
            Self::Yellow => "Yellow",
            Self::Green => "Green",
            Self::Blue => "Blue",
            Self::Purple => "Purple",
        })
    }
}

impl From<Color> for iced::Color {
    fn from(value: Color) -> Self {
        match value {
            Color::Black => iced::Color::from_rgb8(0, 0, 0),
            Color::Red => iced::Color::from_rgb8(220, 50, 47),
            Color::Orange => iced::Color::from_rgb8(203, 75, 22),
            Color::Yellow => iced::Color::from_rgb8(181, 137, 0),
            Color::Green => iced::Color::from_rgb8(133, 153, 0),
            Color::Blue => iced::Color::from_rgb8(38, 139, 210),
            Color::Purple => iced::Color::from_rgb8(108, 113, 196),
        }
    }
}

#[derive(Clone, Debug, Eq)]
struct Item {
    name: String,
    color: Color,
}

impl Hash for Item {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.name.hash(state);
    }
}

impl PartialEq for Item {
    fn eq(&self, other: &Self) -> bool {
        self.name.eq(&other.name)
    }
}

impl From<&str> for Item {
    fn from(s: &str) -> Self {
        Self {
            name: s.to_owned(),
            color: Color::default(),
        }
    }
}

#[derive(Debug, Clone)]
enum Message {
    InputChanged(String),
    ToggleOrder,
    DeleteItem(Item),
    AddItem(String),
    ItemColorChanged(Item, Color),
}

impl App {
    fn update(&mut self, message: Message) {
        match message {
            Message::InputChanged(input) => {
                self.input = input;
            }
            Message::ToggleOrder => {
                self.version = self.version.wrapping_add(1);
                self.order = match self.order {
                    Order::Ascending => Order::Descending,
                    Order::Descending => Order::Ascending,
                }
            }
            Message::AddItem(name) => {
                self.version = self.version.wrapping_add(1);
                self.items.insert(name.as_str().into());
                self.input.clear();
            }
            Message::DeleteItem(item) => {
                self.version = self.version.wrapping_add(1);
                self.items.remove(&item);
            }
            Message::ItemColorChanged(item, color) => {
                self.version = self.version.wrapping_add(1);
                if self.items.remove(&item) {
                    self.items.insert(Item {
                        name: item.name,
                        color,
                    });
                }
            }
        }
    }

    fn view(&self) -> Element<Message> {
        let options = lazy(self.version, |_| {
            let mut items: Vec<_> = self.items.iter().cloned().collect();

            items.sort_by(|a, b| match self.order {
                Order::Ascending => {
                    a.name.to_lowercase().cmp(&b.name.to_lowercase())
                }
                Order::Descending => {
                    b.name.to_lowercase().cmp(&a.name.to_lowercase())
                }
            });

            column(items.into_iter().map(|item| {
                let button = button("Delete")
                    .on_press(Message::DeleteItem(item.clone()))
                    .style(button::danger);

                row![
                    text(item.name.clone()).color(item.color),
                    horizontal_space(),
                    pick_list(Color::ALL, Some(item.color), move |color| {
                        Message::ItemColorChanged(item.clone(), color)
                    }),
                    button
                ]
                .spacing(20)
                .into()
            }))
            .spacing(10)
        });

        column![
            scrollable(options).height(Fill),
            row![
                text_input("Add a new option", &self.input)
                    .on_input(Message::InputChanged)
                    .on_submit(Message::AddItem(self.input.clone())),
                button(text!("Toggle Order ({})", self.order))
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
