use super::Renderer;

use ggez::graphics::{self, Color};

pub use iced::{button, slider, Button, Slider};

pub type Text = iced::Text<Color>;
pub type Checkbox<Message> = iced::Checkbox<Color, Message>;
pub type Radio<Message> = iced::Radio<Color, Message>;
pub type Image = iced::Image<graphics::Image>;

pub type Column<'a, Message> = iced::Column<'a, Message, Renderer<'a>>;
pub type Row<'a, Message> = iced::Row<'a, Message, Renderer<'a>>;
pub type Element<'a, Message> = iced::Element<'a, Message, Renderer<'a>>;
