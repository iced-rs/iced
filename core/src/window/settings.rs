//! Configure your windows.
#[cfg(target_os = "windows")]
#[path = "settings/windows.rs"]
mod platform;

#[cfg(target_os = "macos")]
#[path = "settings/macos.rs"]
mod platform;

#[cfg(target_os = "linux")]
#[path = "settings/linux.rs"]
mod platform;

#[cfg(target_arch = "wasm32")]
#[path = "settings/wasm.rs"]
mod platform;

#[cfg(not(any(
    target_os = "windows",
    target_os = "macos",
    target_os = "linux",
    target_arch = "wasm32"
)))]
#[path = "settings/other.rs"]
mod platform;

use crate::Size;
use crate::window::{Icon, Level, Position};

pub use platform::PlatformSpecific;

/// The window settings of an application.
#[derive(Debug, Clone)]
pub struct Settings {
    /// The initial logical dimensions of the window.
    pub size: Size,

    /// Whether the window should start maximized.
    pub maximized: bool,

    /// Whether the window should start fullscreen.
    pub fullscreen: bool,

    /// The initial position of the window.
    pub position: Position,

    /// The minimum size of the window.
    pub min_size: Option<Size>,

    /// The maximum size of the window.
    pub max_size: Option<Size>,

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

    /// Whether the window will close when the user requests it, e.g. when a user presses the
    /// close button.
    ///
    /// This can be useful if you want to have some behavior that executes before the window is
    /// actually destroyed. If you disable this, you must manually close the window with the
    /// `window::close` command.
    ///
    /// By default this is enabled.
    pub exit_on_close_request: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            size: Size::new(1024.0, 768.0),
            maximized: false,
            fullscreen: false,
            position: Position::default(),
            min_size: None,
            max_size: None,
            visible: true,
            resizable: true,
            decorations: true,
            transparent: false,
            level: Level::default(),
            icon: None,
            exit_on_close_request: true,
            platform_specific: PlatformSpecific::default(),
        }
    }
}
