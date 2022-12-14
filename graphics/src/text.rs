//! Draw text for your users.
pub mod font;

pub use iced_native::text::{self, *};

use std::cell::RefCell;

#[cfg(feature = "canvas")]
/// An access to system fonts.
pub static FONT_SYSTEM: once_cell::sync::Lazy<cosmic_text::FontSystem> =
    once_cell::sync::Lazy::new(|| cosmic_text::FontSystem::new());

/// A text cache.
#[allow(missing_debug_implementations)]
pub struct Cache {
    #[cfg(feature = "canvas")]
    pub(crate) swash: RefCell<cosmic_text::SwashCache<'static>>,
}

impl Cache {
    /// Creates a new text [`Cache`].
    pub fn new() -> Self {
        Self {
            #[cfg(feature = "canvas")]
            swash: RefCell::new(cosmic_text::SwashCache::new(&FONT_SYSTEM)),
        }
    }
}
