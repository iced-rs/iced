//! Load and use fonts.
use crate::Action;
use crate::core::font::Error;
use crate::task::{self, Task};

use std::borrow::Cow;

/// Load a font from its bytes.
pub fn load(bytes: impl Into<Cow<'static, [u8]>>) -> Task<Result<(), Error>> {
    task::oneshot(|channel| Action::LoadFont {
        bytes: bytes.into(),
        channel,
    })
}

/// Lists all the available font families in the system.
pub fn list() -> Task<Result<Vec<String>, Error>> {
    task::oneshot(|channel| Action::ListFonts { channel })
}
