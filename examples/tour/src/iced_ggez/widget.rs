use super::Renderer;

pub use iced_native::{
    button, slider, text, Align, Button, Checkbox, Color, Length, Radio,
    Slider, Text,
};

pub type Image<'a> = iced_native::Image<&'a str>;

pub type Column<'a, Message> = iced_native::Column<'a, Message, Renderer<'a>>;
pub type Row<'a, Message> = iced_native::Row<'a, Message, Renderer<'a>>;
pub type Element<'a, Message> = iced_native::Element<'a, Message, Renderer<'a>>;
