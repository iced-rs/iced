#![allow(unused)]
use crate::core::Font;
use crate::program;
use crate::widget::{Text, text};

pub const FONT: &[u8] = include_bytes!("../fonts/iced_tester-icons.ttf");

pub fn cancel<'a, Theme, Renderer>() -> Text<'a, Theme, Renderer>
where
    Theme: text::Catalog + 'a,
    Renderer: program::Renderer,
{
    icon("\u{2715}")
}

pub fn check<'a, Theme, Renderer>() -> Text<'a, Theme, Renderer>
where
    Theme: text::Catalog + 'a,
    Renderer: program::Renderer,
{
    icon("\u{2713}")
}

pub fn floppy<'a, Theme, Renderer>() -> Text<'a, Theme, Renderer>
where
    Theme: text::Catalog + 'a,
    Renderer: program::Renderer,
{
    icon("\u{1F4BE}")
}

pub fn folder<'a, Theme, Renderer>() -> Text<'a, Theme, Renderer>
where
    Theme: text::Catalog + 'a,
    Renderer: program::Renderer,
{
    icon("\u{1F4C1}")
}

pub fn keyboard<'a, Theme, Renderer>() -> Text<'a, Theme, Renderer>
where
    Theme: text::Catalog + 'a,
    Renderer: program::Renderer,
{
    icon("\u{2328}")
}

pub fn lightbulb<'a, Theme, Renderer>() -> Text<'a, Theme, Renderer>
where
    Theme: text::Catalog + 'a,
    Renderer: program::Renderer,
{
    icon("\u{F0EB}")
}

pub fn mouse_pointer<'a, Theme, Renderer>() -> Text<'a, Theme, Renderer>
where
    Theme: text::Catalog + 'a,
    Renderer: program::Renderer,
{
    icon("\u{F245}")
}

pub fn pause<'a, Theme, Renderer>() -> Text<'a, Theme, Renderer>
where
    Theme: text::Catalog + 'a,
    Renderer: program::Renderer,
{
    icon("\u{2389}")
}

pub fn pencil<'a, Theme, Renderer>() -> Text<'a, Theme, Renderer>
where
    Theme: text::Catalog + 'a,
    Renderer: program::Renderer,
{
    icon("\u{270E}")
}

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

pub fn tape<'a, Theme, Renderer>() -> Text<'a, Theme, Renderer>
where
    Theme: text::Catalog + 'a,
    Renderer: program::Renderer,
{
    icon("\u{2707}")
}

fn icon<'a, Theme, Renderer>(codepoint: &'a str) -> Text<'a, Theme, Renderer>
where
    Theme: text::Catalog + 'a,
    Renderer: program::Renderer,
{
    text(codepoint).font(Font::with_name("iced_devtools-icons"))
}
