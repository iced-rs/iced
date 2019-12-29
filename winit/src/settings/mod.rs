//! Configure your application.
use crate::Color;

#[cfg(target_os = "windows")]
#[path = "windows.rs"]
mod platform;
#[cfg(not(target_os = "windows"))]
#[path = "not_windows.rs"]
mod platform;

pub use platform::PlatformSpecific;

/// The settings of an application.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Settings {
    /// The [`Window`] settings
    ///
    /// [`Window`]: struct.Window.html
    pub window: Window,

    /// The default background color of the application
    pub background_color: Color,
}

impl Default for Settings {
    fn default() -> Settings {
        Settings {
            window: Window::default(),
            background_color: Color::WHITE,
        }
    }
}

/// The window settings of an application.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Window {
    /// The size of the window.
    pub size: (u32, u32),

    /// Whether the window should be resizable or not.
    pub resizable: bool,

    /// Whether the window should have a border, a title bar, etc.
    pub decorations: bool,

    /// Platform specific settings.
    pub platform_specific: platform::PlatformSpecific,
}

impl Default for Window {
    fn default() -> Window {
        Window {
            size: (1024, 768),
            resizable: true,
            decorations: true,
            platform_specific: Default::default(),
        }
    }
}
