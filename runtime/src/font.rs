//! Load and use fonts.
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
