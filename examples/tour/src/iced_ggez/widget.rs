use super::Renderer;

pub use iced::{
    button, slider, text, Align, Button, Checkbox, Color, Radio, Slider, Text,
};

pub type Image<'a> = iced::Image<&'a str>;

pub type Column<'a, Message> = iced::Column<'a, Message, Renderer<'a>>;
pub type Row<'a, Message> = iced::Row<'a, Message, Renderer<'a>>;
pub type Element<'a, Message> = iced::Element<'a, Message, Renderer<'a>>;
