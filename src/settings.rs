//! Configure your application.
use crate::Color;

/// The settings of an application.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Settings {
    /// The [`Window`] settings.
    ///
    /// They will be ignored on the Web.
    ///
    /// [`Window`]: struct.Window.html
    pub window: Window,

    /// The default background [`Color`] of the application
    ///
    /// [`Color`]: ../struct.Color.html
    pub background: Color,

    // TODO: Add `name` for web compatibility
    pub default_font: Option<&'static [u8]>,
}

impl Default for Settings {
    fn default() -> Settings {
        Settings {
            window: Window::default(),
            background: Color::WHITE,
            default_font: None,
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

    /// Whether the window should have a border, a title bar, etc. or not.
    pub decorations: bool,
}

impl Default for Window {
    fn default() -> Window {
        Window {
            size: (1024, 768),
            resizable: true,
            decorations: true,
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl From<Settings> for iced_winit::Settings {
    fn from(settings: Settings) -> iced_winit::Settings {
        iced_winit::Settings {
            window: iced_winit::settings::Window {
                size: settings.window.size,
                resizable: settings.window.resizable,
                decorations: settings.window.decorations,
                platform_specific: Default::default(),
            },
            background: settings.background,
        }
    }
}
