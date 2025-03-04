use iced::widget::{text, Text};
use iced::Font;

pub const FONT_BYTES: &[u8] = include_bytes!("../fonts/changelog-icons.ttf");

const FONT: Font = Font::with_name("changelog-icons");

pub fn copy() -> Text<'static> {
    text('\u{e800}').font(FONT)
}
