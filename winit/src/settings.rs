//! Configure your application.
#[cfg(target_os = "windows")]
#[path = "settings/windows.rs"]
mod platform;

#[cfg(target_os = "macos")]
#[path = "settings/macos.rs"]
mod platform;

#[cfg(not(any(target_os = "windows", target_os = "macos")))]
#[path = "settings/other.rs"]
mod platform;

pub use platform::PlatformSpecific;

use crate::conversion;
use crate::window_configurator::WindowConfigurator;
use crate::winit::event_loop::EventLoopWindowTarget;
use crate::winit::monitor::MonitorHandle;
use crate::{Mode, Position};
use std::fmt::Debug;
use winit::window::WindowBuilder;

/// The settings of an application.
#[derive(Debug, Clone, Default)]
pub struct Settings<Flags> {
    /// The identifier of the application.
    ///
    /// If provided, this identifier may be used to identify the application or
    /// communicate with it through the windowing system.
    pub id: Option<String>,

    /// The [`Window`] settings
    pub window: Window,

    /// The data needed to initialize an [`Application`].
    ///
    /// [`Application`]: crate::Application
    pub flags: Flags,

    /// Whether the [`Application`] should exit when the user requests the
    /// window to close (e.g. the user presses the close button).
    pub exit_on_close_request: bool,
}

/// The window settings of an application.
#[derive(Debug, Clone)]
pub struct Window {
    /// The size of the window.
    pub size: (u32, u32),

    /// The position of the window.
    pub position: Position,

    /// The minimum size of the window.
    pub min_size: Option<(u32, u32)>,

    /// The maximum size of the window.
    pub max_size: Option<(u32, u32)>,

    /// Whether the window should be resizable or not.
    pub resizable: bool,

    /// Whether the window should have a border, a title bar, etc.
    pub decorations: bool,

    /// Whether the window should be transparent.
    pub transparent: bool,

    /// Whether the window will always be on top of other windows.
    pub always_on_top: bool,

    /// The window icon, which is also usually used in the taskbar
    pub icon: Option<winit::window::Icon>,

    /// Platform specific settings.
    pub platform_specific: platform::PlatformSpecific,
}

impl Window {
    /// Converts the window settings into a `WindowBuilder` from `winit`.
    pub fn into_builder(
        self,
        title: &str,
        mode: Mode,
        primary_monitor: Option<MonitorHandle>,
        _id: Option<String>,
    ) -> WindowBuilder {
        let builder = WindowBuilder::new().with_title(title);
        configure_builder(self, primary_monitor, mode, _id, builder)
    }
}

/// Default `WindowConfigurator` that configures the winit WindowBuilder
#[derive(Debug)]
pub struct SettingsWindowConfigurator {
    /// base window settings
    pub window: Window,

    /// Optional application id
    pub id: Option<String>,

    /// initial window mode
    pub mode: Mode,
}

impl<A> WindowConfigurator<A> for SettingsWindowConfigurator {
    fn configure_builder(
        self,
        available_monitors: &EventLoopWindowTarget<A>,
        window_builder: WindowBuilder,
    ) -> WindowBuilder {
        let selected_monitor = available_monitors.primary_monitor();
        configure_builder(
            self.window,
            selected_monitor,
            self.mode,
            self.id,
            window_builder,
        )
    }
}

impl Default for Window {
    fn default() -> Window {
        Window {
            size: (1024, 768),
            position: Position::default(),
            min_size: None,
            max_size: None,
            resizable: true,
            decorations: true,
            transparent: false,
            always_on_top: false,
            icon: None,
            platform_specific: Default::default(),
        }
    }
}

fn configure_builder(
    window: Window,
    selected_monitor: Option<MonitorHandle>,
    mode: Mode,
    _id: Option<String>,
    mut window_builder: WindowBuilder,
) -> WindowBuilder {
    let (width, height) = window.size;

    window_builder = window_builder
        .with_inner_size(winit::dpi::LogicalSize { width, height })
        .with_resizable(window.resizable)
        .with_decorations(window.decorations)
        .with_transparent(window.transparent)
        .with_window_icon(window.icon)
        .with_always_on_top(window.always_on_top)
        .with_visible(conversion::visible(mode));

    if let Some(position) = conversion::position(
        selected_monitor.as_ref(),
        window.size,
        window.position,
    ) {
        window_builder = window_builder.with_position(position);
    }

    if let Some((width, height)) = window.min_size {
        window_builder = window_builder
            .with_min_inner_size(winit::dpi::LogicalSize { width, height });
    }

    if let Some((width, height)) = window.max_size {
        window_builder = window_builder
            .with_max_inner_size(winit::dpi::LogicalSize { width, height });
    }

    #[cfg(any(
        target_os = "linux",
        target_os = "dragonfly",
        target_os = "freebsd",
        target_os = "netbsd",
        target_os = "openbsd"
    ))]
    {
        use ::winit::platform::unix::WindowBuilderExtUnix;

        if let Some(id) = _id {
            window_builder = window_builder.with_app_id(id);
        }
    }

    #[cfg(target_os = "windows")]
    {
        use winit::platform::windows::WindowBuilderExtWindows;

        if let Some(parent) = window.platform_specific.parent {
            window_builder = window_builder.with_parent_window(parent);
        }

        window_builder = window_builder
            .with_drag_and_drop(window.platform_specific.drag_and_drop);
    }

    #[cfg(target_os = "macos")]
    {
        use winit::platform::macos::WindowBuilderExtMacOS;

        window_builder = window_builder
            .with_title_hidden(window.platform_specific.title_hidden)
            .with_titlebar_transparent(
                window.platform_specific.titlebar_transparent,
            )
            .with_fullsize_content_view(
                window.platform_specific.fullsize_content_view,
            );
    }

    window_builder = window_builder
        .with_fullscreen(conversion::fullscreen(selected_monitor, mode));

    window_builder
}
