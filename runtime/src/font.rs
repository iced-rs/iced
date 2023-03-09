//! Load and use fonts.
pub use iced_core::font::*;

use crate::command::{self, Command};
use std::borrow::Cow;

/// An error while loading a font.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {}

/// Load a font from its bytes.
pub fn load(
    bytes: impl Into<Cow<'static, [u8]>>,
) -> Command<Result<(), Error>> {
    Command::single(command::Action::LoadFont {
        bytes: bytes.into(),
        tagger: Box::new(std::convert::identity),
    })
}
