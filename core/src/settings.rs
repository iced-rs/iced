//! Configure your application.
use crate::backend;
use crate::renderer;
use crate::text::LineHeight;
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
    pub font: Font,

    /// The text size that will be used by default.
    ///
    /// By default, it is `16.0`.
    pub text_size: Pixels,

    /// The default line height of text.
    ///
    /// By default, it is `LineHeight::Relative(1.375)`.
    pub line_height: LineHeight,

    /// Whether certain widgets should be rendered using metrics hinting.
    ///
    /// Metrics hinting can improve the readability of smaller text in
    /// low-DPI screens, as well as the clarity of widgets that render thin lines.
    ///
    /// By default, it is enabled.
    pub metrics_hinting: bool,

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
            font: renderer.font,
            text_size: renderer.text_size,
            line_height: renderer.line_height,
            metrics_hinting: true,
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
            font: settings.font,
            text_size: settings.text_size,
            line_height: settings.line_height,
            metrics_hinting: settings.metrics_hinting,
        }
    }
}
