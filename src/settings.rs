//! Configure your application.
use crate::{window, Color};

/// The settings of an application.
#[derive(Debug, Clone, Copy, PartialEq)]
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

    /// The background color of the window.
    ///
    /// On supported backends, this makes it possible to have
    /// (semi-)transparent windows.
    ///
    /// By default, it is white.
    pub background_color: Color,
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
            window: Default::default(),
            background_color: Color::WHITE,
        }
    }
}

impl<Flags> Settings<Flags> {
    /// Initialize application settings using the given data.
    ///
    /// [`Application`]: ../trait.Application.html
    pub fn with_flags(flags: Flags) -> Self {
        Self {
            flags,
            // not using ..Default::default() struct update syntax since it is more permissive to
            // allow initializing with flags without trait bound on Default
            antialiasing: Default::default(),
            default_font: Default::default(),
            window: Default::default(),
            background_color: Color::WHITE,
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl<Flags> From<Settings<Flags>> for iced_winit::Settings<Flags> {
    fn from(settings: Settings<Flags>) -> iced_winit::Settings<Flags> {
        iced_winit::Settings {
            window: iced_winit::settings::Window {
                size: settings.window.size,
                resizable: settings.window.resizable,
                decorations: settings.window.decorations,
                platform_specific: Default::default(),
            },
            flags: settings.flags,
        }
    }
}
