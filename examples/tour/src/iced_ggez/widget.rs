use super::Renderer;

pub use iced::{button, slider, text, Align, Button, Color, Slider};

pub type Text = iced::Text<Color>;
pub type Checkbox<Message> = iced::Checkbox<Color, Message>;
pub type Radio<Message> = iced::Radio<Color, Message>;
pub type Image<'a> = iced::Image<&'a str>;

pub type Column<'a, Message> = iced::Column<'a, Message, Renderer<'a>>;
pub type Row<'a, Message> = iced::Row<'a, Message, Renderer<'a>>;
pub type Element<'a, Message> = iced::Element<'a, Message, Renderer<'a>>;
