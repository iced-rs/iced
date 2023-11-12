//! Configure your application.
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

pub use platform::PlatformSpecific;

use crate::conversion;
use crate::core::window::{Icon, Level};
use crate::Position;

use winit::monitor::MonitorHandle;
use winit::window::WindowBuilder;

use std::borrow::Cow;
use std::fmt;

/// The settings of an application.
#[derive(Debug, Clone, Default)]
pub struct Settings<Flags> {
    /// The identifier of the application.
    ///
    /// If provided, this identifier may be used to identify the application or
    /// communicate with it through the windowing system.
    pub id: Option<String>,

    /// The [`Window`] settings.
    pub window: Window,

    /// The data needed to initialize an [`Application`].
    ///
    /// [`Application`]: crate::Application
    pub flags: Flags,

    /// The fonts to load on boot.
    pub fonts: Vec<Cow<'static, [u8]>>,

    /// Whether the [`Application`] should exit when the user requests the
    /// window to close (e.g. the user presses the close button).
    ///
    /// [`Application`]: crate::Application
    pub exit_on_close_request: bool,
}

/// The window settings of an application.
#[derive(Clone)]
pub struct Window {
    /// The size of the window.
    pub size: (u32, u32),

    /// The position of the window.
    pub position: Position,

    /// The minimum size of the window.
    pub min_size: Option<(u32, u32)>,

    /// The maximum size of the window.
    pub max_size: Option<(u32, u32)>,

    /// Whether the window should be visible or not.
    pub visible: bool,

    /// Whether the window should be resizable or not.
    pub resizable: bool,

    /// Whether the window should have a border, a title bar, etc.
    pub decorations: bool,

    /// Whether the window should be transparent.
    pub transparent: bool,

    /// The window [`Level`].
    pub level: Level,

    /// The window icon, which is also usually used in the taskbar
    pub icon: Option<Icon>,

    /// Platform specific settings.
    pub platform_specific: platform::PlatformSpecific,
}

impl fmt::Debug for Window {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Window")
            .field("size", &self.size)
            .field("position", &self.position)
            .field("min_size", &self.min_size)
            .field("max_size", &self.max_size)
            .field("visible", &self.visible)
            .field("resizable", &self.resizable)
            .field("decorations", &self.decorations)
            .field("transparent", &self.transparent)
            .field("level", &self.level)
            .field("icon", &self.icon.is_some())
            .field("platform_specific", &self.platform_specific)
            .finish()
    }
}

impl Window {
    /// Converts the window settings into a `WindowBuilder` from `winit`.
    pub fn into_builder(
        self,
        title: &str,
        primary_monitor: Option<MonitorHandle>,
        _id: Option<String>,
    ) -> WindowBuilder {
        let mut window_builder = WindowBuilder::new();

        let (width, height) = self.size;

        window_builder = window_builder
            .with_title(title)
            .with_inner_size(winit::dpi::LogicalSize { width, height })
            .with_resizable(self.resizable)
            .with_enabled_buttons(if self.resizable {
                winit::window::WindowButtons::all()
            } else {
                winit::window::WindowButtons::CLOSE
                    | winit::window::WindowButtons::MINIMIZE
            })
            .with_decorations(self.decorations)
            .with_transparent(self.transparent)
            .with_window_icon(self.icon.and_then(conversion::icon))
            .with_window_level(conversion::window_level(self.level))
            .with_visible(self.visible);

        if let Some(position) = conversion::position(
            primary_monitor.as_ref(),
            self.size,
            self.position,
        ) {
            window_builder = window_builder.with_position(position);
        }

        if let Some((width, height)) = self.min_size {
            window_builder = window_builder
                .with_min_inner_size(winit::dpi::LogicalSize { width, height });
        }

        if let Some((width, height)) = self.max_size {
            window_builder = window_builder
                .with_max_inner_size(winit::dpi::LogicalSize { width, height });
        }

        #[cfg(any(
            target_os = "dragonfly",
            target_os = "freebsd",
            target_os = "netbsd",
            target_os = "openbsd"
        ))]
        {
            // `with_name` is available on both `WindowBuilderExtWayland` and `WindowBuilderExtX11` and they do
            // exactly the same thing. We arbitrarily choose `WindowBuilderExtWayland` here.
            use ::winit::platform::wayland::WindowBuilderExtWayland;

            if let Some(id) = _id {
                window_builder = window_builder.with_name(id.clone(), id);
            }
        }

        #[cfg(target_os = "windows")]
        {
            use winit::platform::windows::WindowBuilderExtWindows;
            #[allow(unsafe_code)]
            unsafe {
                window_builder = window_builder
                    .with_parent_window(self.platform_specific.parent);
            }
            window_builder = window_builder
                .with_drag_and_drop(self.platform_specific.drag_and_drop);
        }

        #[cfg(target_os = "macos")]
        {
            use winit::platform::macos::WindowBuilderExtMacOS;

            window_builder = window_builder
                .with_title_hidden(self.platform_specific.title_hidden)
                .with_titlebar_transparent(
                    self.platform_specific.titlebar_transparent,
                )
                .with_fullsize_content_view(
                    self.platform_specific.fullsize_content_view,
                );
        }

        #[cfg(target_os = "linux")]
        {
            #[cfg(feature = "x11")]
            {
                use winit::platform::x11::WindowBuilderExtX11;

                window_builder = window_builder.with_name(
                    &self.platform_specific.application_id,
                    &self.platform_specific.application_id,
                );
            }
            #[cfg(feature = "wayland")]
            {
                use winit::platform::wayland::WindowBuilderExtWayland;

                window_builder = window_builder.with_name(
                    &self.platform_specific.application_id,
                    &self.platform_specific.application_id,
                );
            }
        }

        window_builder
    }
}

impl Default for Window {
    fn default() -> Window {
        Window {
            size: (1024, 768),
            position: Position::default(),
            min_size: None,
            max_size: None,
            visible: true,
            resizable: true,
            decorations: true,
            transparent: false,
            level: Level::default(),
            icon: None,
            platform_specific: PlatformSpecific::default(),
        }
    }
}
