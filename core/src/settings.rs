//! Configure your application.
use crate::backend;
use crate::renderer;
use crate::{Backend, Font, Pixels};

use std::borrow::Cow;

/// The settings of an iced program.
#[derive(Debug, Clone)]
pub struct Settings {
    /// The identifier of the application.
    ///
    /// If provided, this identifier may be used to identify the application or
    /// communicate with it through the windowing system.
    pub id: Option<String>,

    /// The fonts to load on boot.
    pub fonts: Vec<Cow<'static, [u8]>>,

    /// The default [`Font`] to be used.
    ///
    /// By default, it uses [`Family::SansSerif`](crate::font::Family::SansSerif).
    pub default_font: Font,

    /// The text size that will be used by default.
    ///
    /// By default, it is `16.0`.
    pub default_text_size: Pixels,

    /// The graphical backend to use.
    ///
    /// By default, it is [`Backend::Best`].
    pub backend: Backend,

    /// The [`PowerPreference`](backend::PowerPreference) of the backend.
    ///
    /// By default, it is [`backend::PowerPreference::None`].
    pub power_preference: backend::PowerPreference,

    /// If set to true, the renderer will try to perform antialiasing for some
    /// primitives.
    ///
    /// Enabling it can produce a smoother result in some widgets, like the
    /// `canvas` widget, at a performance cost.
    ///
    /// By default, it is enabled.
    pub antialiasing: bool,

    /// Whether or not to attempt to synchronize rendering when possible.
    ///
    /// Disabling it can improve rendering performance on some platforms.
    ///
    /// By default, it is enabled.
    pub vsync: bool,
}

impl Default for Settings {
    fn default() -> Self {
        let renderer = renderer::Settings::default();

        Self {
            id: None,
            fonts: Vec::new(),
            default_font: renderer.default_font,
            default_text_size: renderer.default_text_size,
            backend: Backend::default(),
            power_preference: backend::PowerPreference::None,
            antialiasing: true,
            vsync: true,
        }
    }
}

impl From<&Settings> for renderer::Settings {
    fn from(settings: &Settings) -> Self {
        Self {
            default_font: settings.default_font,
            default_text_size: settings.default_text_size,
        }
    }
}
