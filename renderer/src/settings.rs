use crate::core::{Font, Pixels};
use crate::graphics::Antialiasing;

/// The settings of a Backend.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Settings {
    /// The default [`Font`] to use.
    pub default_font: Font,

    /// The default size of text.
    ///
    /// By default, it will be set to `16.0`.
    pub default_text_size: Pixels,

    /// The antialiasing strategy that will be used for triangle primitives.
    ///
    /// By default, it is `None`.
    pub antialiasing: Option<Antialiasing>,
}

impl Default for Settings {
    fn default() -> Settings {
        Settings {
            default_font: Font::default(),
            default_text_size: Pixels(16.0),
            antialiasing: None,
        }
    }
}

#[cfg(feature = "tiny-skia")]
impl From<Settings> for iced_tiny_skia::Settings {
    fn from(settings: Settings) -> Self {
        Self {
            default_font: settings.default_font,
            default_text_size: settings.default_text_size,
        }
    }
}

#[cfg(feature = "wgpu")]
impl From<Settings> for iced_wgpu::Settings {
    fn from(settings: Settings) -> Self {
        Self {
            default_font: settings.default_font,
            default_text_size: settings.default_text_size,
            antialiasing: settings.antialiasing,
            ..iced_wgpu::Settings::default()
        }
    }
}

impl From<Settings> for () {
    fn from(_settings: Settings) -> Self {}
}
