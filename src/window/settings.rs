use crate::window::{Icon, Level, Position,WindowTheme};

pub use iced_winit::settings::PlatformSpecific;

/// The window settings of an application.
#[derive(Debug, Clone)]
pub struct Settings {
    /// The initial size of the window.
    pub size: (u32, u32),

    /// The initial position of the window.
    pub position: Position,

    /// The minimum size of the window.
    pub min_size: Option<(u32, u32)>,

    /// The maximum size of the window.
    pub max_size: Option<(u32, u32)>,

    /// Whether the window should be visible or not.
    pub visible: bool,

    /// Whether the window should be resizable or not.
    pub resizable: bool,

    /// Whether the window should have a border, a title bar, etc. or not.
    pub decorations: bool,
    /// Sets a specific theme for the window.
    ///
    /// If `None` is provided, the window will use the system theme.
    ///
    /// The default is `None`.
    ///
    /// ## Platform-specific
    ///
    /// - **macOS:** This is an app-wide setting.
    /// - **Wayland:** This control only CSD. You can also use `WINIT_WAYLAND_CSD_THEME` env variable to set the theme.
    ///   Possible values for env variable are: "dark" and light".
    /// - **x11:** Build window with `_GTK_THEME_VARIANT` hint set to `dark` or `light`.
    /// - **iOS / Android / Web / x11 / Orbital:** Ignored.
    pub window_theme: Option<WindowTheme>,
    /// Whether the window should be transparent.
    pub transparent: bool,

    /// The window [`Level`].
    pub level: Level,

    /// The icon of the window.
    pub icon: Option<Icon>,

    /// Platform specific settings.
    pub platform_specific: PlatformSpecific,
}

impl Default for Settings {
    fn default() -> Settings {
        Settings {
            size: (1024, 768),
            position: Position::default(),
            min_size: None,
            max_size: None,
            visible: true,
            resizable: true,
            decorations: true,
            window_theme: None,
            transparent: false,
            level: Level::default(),
            icon: None,
            platform_specific: Default::default(),
        }
    }
}

impl From<Settings> for iced_winit::settings::Window {
    fn from(settings: Settings) -> Self {
        Self {
            size: settings.size,
            position: iced_winit::Position::from(settings.position),
            min_size: settings.min_size,
            max_size: settings.max_size,
            visible: settings.visible,
            resizable: settings.resizable,
            decorations: settings.decorations,
            window_theme: settings.window_theme,
            transparent: settings.transparent,
            level: settings.level,
            icon: settings.icon.map(Icon::into),
            platform_specific: settings.platform_specific,
        }
    }
}
