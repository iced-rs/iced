use ggez::graphics::Color;

pub use iced::{button, Button, Column, Row};

pub type Text = iced::Text<Color>;
pub type Checkbox<Message> = iced::Checkbox<Color, Message>;
