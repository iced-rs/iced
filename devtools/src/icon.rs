// Generated automatically by iced_fontello at build time.
// Do not edit manually. Source: ../fonts/iced_devtools-icons.toml
// 3139a163a989c992b8f038da359b59e9292fc49f031e760b61a8d76e2037aee2
use crate::core::Font;
use crate::program;
use crate::widget::{Text, text};

pub const FONT: &[u8] = include_bytes!("../fonts/iced_devtools-icons.ttf");

pub fn play<'a, Theme, Renderer>() -> Text<'a, Theme, Renderer>
where
    Theme: text::Catalog + 'a,
    Renderer: program::Renderer,
{
    icon("\u{25B6}")
}

pub fn record<'a, Theme, Renderer>() -> Text<'a, Theme, Renderer>
where
    Theme: text::Catalog + 'a,
    Renderer: program::Renderer,
{
    icon("\u{26AB}")
}

pub fn stop<'a, Theme, Renderer>() -> Text<'a, Theme, Renderer>
where
    Theme: text::Catalog + 'a,
    Renderer: program::Renderer,
{
    icon("\u{25A0}")
}

fn icon<'a, Theme, Renderer>(codepoint: &'a str) -> Text<'a, Theme, Renderer>
where
    Theme: text::Catalog + 'a,
    Renderer: program::Renderer,
{
    text(codepoint).font(Font::with_name("iced_devtools-icons"))
}
