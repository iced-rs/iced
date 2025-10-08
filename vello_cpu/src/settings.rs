use crate::core::{Font, Pixels};
use crate::graphics;

/// The settings of a [`Compositor`].
///
/// [`Compositor`]: crate::window::Compositor
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Settings {
    /// The default [`Font`] to use.
    pub default_font: Font,

    /// The default size of text.
    ///
    /// By default, it will be set to `16.0`.
    pub default_text_size: Pixels,
}

impl Default for Settings {
    fn default() -> Settings {
        Settings {
            default_font: Font::default(),
            default_text_size: Pixels(16.0),
        }
    }
}

impl From<graphics::Settings> for Settings {
    fn from(settings: graphics::Settings) -> Self {
        Self {
            default_font: settings.default_font,
            default_text_size: settings.default_text_size,
        }
    }
}
