#![allow(unused)]
use crate::core::Font;
use crate::widget::{Text, text};

pub const FONT: &[u8] = include_bytes!("../fonts/iced_tester-icons.ttf");

pub fn cancel<'a, Theme>() -> Text<'a, Theme>
where
    Theme: text::Catalog + 'a,
{
    icon("\u{2715}")
}

pub fn check<'a, Theme>() -> Text<'a, Theme>
where
    Theme: text::Catalog + 'a,
{
    icon("\u{2713}")
}

pub fn floppy<'a, Theme>() -> Text<'a, Theme>
where
    Theme: text::Catalog + 'a,
{
    icon("\u{1F4BE}")
}

pub fn folder<'a, Theme>() -> Text<'a, Theme>
where
    Theme: text::Catalog + 'a,
{
    icon("\u{1F4C1}")
}

pub fn keyboard<'a, Theme>() -> Text<'a, Theme>
where
    Theme: text::Catalog + 'a,
{
    icon("\u{2328}")
}

pub fn lightbulb<'a, Theme>() -> Text<'a, Theme>
where
    Theme: text::Catalog + 'a,
{
    icon("\u{F0EB}")
}

pub fn mouse_pointer<'a, Theme>() -> Text<'a, Theme>
where
    Theme: text::Catalog + 'a,
{
    icon("\u{F245}")
}

pub fn pause<'a, Theme>() -> Text<'a, Theme>
where
    Theme: text::Catalog + 'a,
{
    icon("\u{2389}")
}

pub fn pencil<'a, Theme>() -> Text<'a, Theme>
where
    Theme: text::Catalog + 'a,
{
    icon("\u{270E}")
}

pub fn play<'a, Theme>() -> Text<'a, Theme>
where
    Theme: text::Catalog + 'a,
{
    icon("\u{25B6}")
}

pub fn record<'a, Theme>() -> Text<'a, Theme>
where
    Theme: text::Catalog + 'a,
{
    icon("\u{26AB}")
}

pub fn stop<'a, Theme>() -> Text<'a, Theme>
where
    Theme: text::Catalog + 'a,
{
    icon("\u{25A0}")
}

pub fn tape<'a, Theme>() -> Text<'a, Theme>
where
    Theme: text::Catalog + 'a,
{
    icon("\u{2707}")
}

fn icon<'a, Theme>(codepoint: &'a str) -> Text<'a, Theme>
where
    Theme: text::Catalog + 'a,
{
    text(codepoint).font("iced_devtools-icons")
}
