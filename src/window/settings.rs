use crate::window::Icon;

/// The window settings of an application.
#[derive(Debug, Clone)]
pub struct Settings {
    /// The initial size of the window.
    pub size: (u32, u32),

    /// The initial position of the window.
    ///
    /// When the decorations of the window are enabled, Windows 10 will add some inivisble padding
    /// to the window. This padding gets included in the position. So if you have decorations
    /// enabled and want the window to be at 0,0 you would have to set the position to
    /// -DPI_BORDER_X,-DPI_BORDER_Y.
    ///
    /// DPI_BORDER_X/DPI_BORDER_Y are the usual size of the padding, which changes based on the DPI of the display.
    ///
    /// On a 1920x1080 monitor you would have to set the position to -8,-2.
    ///
    /// For info on how you could implement positioning that supports all DPI monitors look at the
    /// following WINAPI calls:
    ///
    /// * GetDpiForMonitor (with MDT_RAW_DPI)
    /// * GetSystemMetricsForDpi (with SM_CXFRAME and SM_CYFRAME)
    ///     
    /// Note: this gets ignored on the web
    pub position: (i32, i32),

    /// The minimum size of the window.
    pub min_size: Option<(u32, u32)>,

    /// The maximum size of the window.
    pub max_size: Option<(u32, u32)>,

    /// Whether the window should be resizable or not.
    pub resizable: bool,

    /// Whether the window should have a border, a title bar, etc. or not.
    pub decorations: bool,

    /// Whether the window should be transparent.
    pub transparent: bool,

    /// Whether the window will always be on top of other windows.
    pub always_on_top: bool,

    /// The icon of the window.
    pub icon: Option<Icon>,
}

impl Default for Settings {
    fn default() -> Settings {
        Settings {
            size: (1024, 768),
            position: (100, 100),
            min_size: None,
            max_size: None,
            resizable: true,
            decorations: true,
            transparent: false,
            always_on_top: false,
            icon: None,
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl From<Settings> for iced_winit::settings::Window {
    fn from(settings: Settings) -> Self {
        Self {
            size: settings.size,
            position: settings.position,
            min_size: settings.min_size,
            max_size: settings.max_size,
            resizable: settings.resizable,
            decorations: settings.decorations,
            transparent: settings.transparent,
            always_on_top: settings.always_on_top,
            icon: settings.icon.map(Icon::into),
            platform_specific: Default::default(),
        }
    }
}
