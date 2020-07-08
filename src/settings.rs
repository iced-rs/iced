//! Configure your application.
use crate::window;

/// The settings of an application.
#[derive(Debug, Clone)]
pub struct Settings<Flags> {
    /// The window settings.
    ///
    /// They will be ignored on the Web.
    ///
    /// [`Window`]: struct.Window.html
    pub window: window::Settings,

    /// The data needed to initialize an [`Application`].
    ///
    /// [`Application`]: ../trait.Application.html
    pub flags: Flags,

    /// The bytes of the font that will be used by default.
    ///
    /// If `None` is provided, a default system font will be chosen.
    // TODO: Add `name` for web compatibility
    pub default_font: Option<&'static [u8]>,

    /// The text size that will be used by default.
    ///
    /// The default value is 20.
    pub default_text_size: u16,

    /// If set to true, the renderer will try to perform antialiasing for some
    /// primitives.
    ///
    /// Enabling it can produce a smoother result in some widgets, like the
    /// [`Canvas`], at a performance cost.
    ///
    /// By default, it is disabled.
    ///
    /// [`Canvas`]: ../widget/canvas/struct.Canvas.html
    pub antialiasing: bool,
}

impl<Flags> Settings<Flags> {
    /// Initialize application settings using the given data.
    ///
    /// [`Application`]: ../trait.Application.html
    pub fn with_flags(flags: Flags) -> Self {
        let default_settings = Settings::<()>::default();

        Self {
            flags,
            antialiasing: default_settings.antialiasing,
            default_font: default_settings.default_font,
            default_text_size: default_settings.default_text_size,
            window: default_settings.window,
        }
    }
}

impl<Flags> Default for Settings<Flags>
where
    Flags: Default,
{
    fn default() -> Self {
        Self {
            flags: Default::default(),
            antialiasing: Default::default(),
            default_font: Default::default(),
            default_text_size: 20,
            window: Default::default(),
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl<Flags> From<Settings<Flags>> for iced_winit::Settings<Flags> {
    fn from(settings: Settings<Flags>) -> iced_winit::Settings<Flags> {
        iced_winit::Settings {
            window: settings.window.into(),
            flags: settings.flags,
        }
    }
}
