//! Configure your application.

#[cfg(target_os = "macos")]
#[path = "settings/macos.rs"]
mod platform;

#[cfg(not(target_os = "macos"))]
#[path = "settings/other.rs"]
mod platform;

use crate::{Font, Pixels};

use std::borrow::Cow;

#[cfg(target_os = "macos")]
pub use platform::ActivationPolicy;
pub use platform::PlatformSpecific;

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
    /// The default value is `16.0`.
    pub default_text_size: Pixels,

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

    /// Platform specific settings.
    pub platform_specific: PlatformSpecific,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            id: None,
            fonts: Vec::new(),
            default_font: Font::default(),
            default_text_size: Pixels(16.0),
            antialiasing: true,
            vsync: true,
            #[allow(clippy::default_constructed_unit_structs)]
            platform_specific: PlatformSpecific::default(),
        }
    }
}
