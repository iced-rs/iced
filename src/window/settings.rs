use crate::window::Position;
#[cfg(feature = "winit")]
use crate::window::Icon;

/// The window settings of an application.
#[derive(Debug, Clone)]
pub struct Settings {
    /// The size of the resize-enabled border.
    pub border_size: u32,

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

    /// Whether the window should be transparent.
    pub transparent: bool,

    /// Whether the window will always be on top of other windows.
    pub always_on_top: bool,

    /// The icon of the window.
    #[cfg(feature = "winit")]
    pub icon: Option<Icon>,
}

impl Default for Settings {
    fn default() -> Settings {
        Settings {
            border_size: 8,
            size: (1024, 768),
            position: Position::default(),
            min_size: None,
            max_size: None,
            visible: true,
            resizable: true,
            decorations: true,
            transparent: false,
            always_on_top: false,
            #[cfg(feature = "winit")]
            icon: None,
        }
    }
}

#[cfg(feature = "winit")]
impl From<Settings> for iced_winit::settings::Window {
    fn from(settings: Settings) -> Self {
        Self {
            border_size: settings.border_size,
            size: settings.size,
            position: iced_winit::Position::from(settings.position),
            min_size: settings.min_size,
            max_size: settings.max_size,
            visible: settings.visible,
            resizable: settings.resizable,
            decorations: settings.decorations,
            transparent: settings.transparent,
            always_on_top: settings.always_on_top,
            icon: settings.icon.map(Icon::into),
            platform_specific: Default::default(),
        }
    }
}
