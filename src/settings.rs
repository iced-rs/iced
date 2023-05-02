//! Configure your application.
use crate::renderer;
use crate::window;

/// The settings of an application.
#[derive(Debug, Clone)]
pub struct Settings<Flags> {
    /// The identifier of the application.
    ///
    /// If provided, this identifier may be used to identify the application or
    /// communicate with it through the windowing system.
    pub id: Option<String>,

    /// The window settings.
    ///
    /// They will be ignored on the Web.
    pub window: window::Settings,

    /// The data needed to initialize the [`Application`].
    ///
    /// [`Application`]: crate::Application
    pub flags: Flags,

    /// The bytes of the font that will be used by default.
    ///
    /// If `None` is provided, a default system font will be chosen.
    // TODO: Add `name` for web compatibility
    pub default_font: Option<&'static [u8]>,

    /// The text size that will be used by default.
    ///
    /// The default value is `20.0`.
    pub default_text_size: f32,

    /// If enabled, spread text workload in multiple threads when multiple cores
    /// are available.
    ///
    /// By default, it is disabled.
    pub text_multithreading: bool,

    /// If set to true, the renderer will try to perform antialiasing for some
    /// primitives.
    ///
    /// Enabling it can produce a smoother result in some widgets, like the
    /// [`Canvas`], at a performance cost.
    ///
    /// By default, it is disabled.
    ///
    /// [`Canvas`]: crate::widget::Canvas
    pub antialiasing: bool,

    /// Whether the [`Application`] should exit when the user requests the
    /// window to close (e.g. the user presses the close button).
    ///
    /// By default, it is enabled.
    ///
    /// [`Application`]: crate::Application
    pub exit_on_close_request: bool,

    /// Whether the [`Application`] should try to build the context
    /// using OpenGL ES first then OpenGL.
    ///
    /// By default, it is disabled.
    /// **Note:** Only works for the `glow` backend.
    ///
    /// [`Application`]: crate::Application
    pub try_opengles_first: bool,

    /// Override renderer settings
    ///
    /// Fields: `default_font`, `default_text_size`, `text_multithreading` and
    /// `antialiasing` of these override settings are ignored
    ///
    /// If `None` is provided, default settings are used
    pub renderer: Option<renderer::Settings>,
}

impl<Flags> Settings<Flags> {
    /// Initialize [`Application`] settings using the given data.
    ///
    /// [`Application`]: crate::Application
    pub fn with_flags(flags: Flags) -> Self {
        let default_settings = Settings::<()>::default();

        Self {
            flags,
            id: default_settings.id,
            window: default_settings.window,
            renderer: default_settings.renderer,
            default_font: default_settings.default_font,
            default_text_size: default_settings.default_text_size,
            text_multithreading: default_settings.text_multithreading,
            antialiasing: default_settings.antialiasing,
            exit_on_close_request: default_settings.exit_on_close_request,
            try_opengles_first: default_settings.try_opengles_first,
        }
    }
}

impl<Flags> Default for Settings<Flags>
where
    Flags: Default,
{
    fn default() -> Self {
        Self {
            id: None,
            window: Default::default(),
            renderer: Default::default(),
            flags: Default::default(),
            default_font: Default::default(),
            default_text_size: 20.0,
            text_multithreading: false,
            antialiasing: false,
            exit_on_close_request: true,
            try_opengles_first: false,
        }
    }
}

impl<Flags> From<Settings<Flags>> for iced_winit::Settings<Flags> {
    fn from(settings: Settings<Flags>) -> iced_winit::Settings<Flags> {
        iced_winit::Settings {
            id: settings.id,
            window: settings.window.into(),
            flags: settings.flags,
            exit_on_close_request: settings.exit_on_close_request,
            try_opengles_first: settings.try_opengles_first,
        }
    }
}
