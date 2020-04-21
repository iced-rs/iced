//! Configure your application.
mod platform {
    /// The platform specific window settings of an application.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
    pub struct PlatformSpecific {}
}
pub use platform::PlatformSpecific;

/// The settings of an application.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Settings<Flags> {
    /// The [`Window`] settings
    ///
    /// [`Window`]: struct.Window.html
    pub window: Window,

    /// The data needed to initialize an [`Application`].
    ///
    /// [`Application`]: trait.Application.html
    pub flags: Flags,
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
