use crate::window::{Icon, Level, Position};

pub use iced_winit::settings::PlatformSpecific;

/// The window settings of an application.
#[derive(Debug, Clone)]
pub struct Settings {
    /// The initial size of the window.
    pub size: (u32, u32),

    /// The border area for the drag resize handle.
    pub resize_border: u32,

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
            resize_border: 8,
            position: Position::default(),
            min_size: None,
            max_size: None,
            visible: true,
            resizable: true,
            decorations: true,
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
            resize_border: settings.resize_border,
            position: iced_winit::Position::from(settings.position),
            min_size: settings.min_size,
            max_size: settings.max_size,
            visible: settings.visible,
            resizable: settings.resizable,
            decorations: settings.decorations,
            transparent: settings.transparent,
            level: settings.level,
            icon: settings.icon.map(Icon::into),
            platform_specific: settings.platform_specific,
        }
    }
}
