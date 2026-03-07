//! Load and use fonts.
use iced_core::Font;

use crate::Action;
use crate::task::{self, Task};
use std::borrow::Cow;

/// An error while loading a font.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {}

/// Load a font from its bytes.
pub fn load(bytes: impl Into<Cow<'static, [u8]>>) -> Task<Result<(), Error>> {
    task::oneshot(|channel| Action::LoadFont {
        bytes: bytes.into(),
        channel,
    })
}

/// Set the default font
pub fn set_default_font(font: impl Into<Font>) -> Task<Result<(), Error>> {
    task::effect(Action::SetDefaultFont(font.into()))
}
