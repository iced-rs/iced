//! Configure your application.
use crate::core::window;
use crate::conversion;

use winit::monitor::MonitorHandle;
use winit::window::WindowBuilder;

/// The settings of an application.
#[derive(Debug, Clone, Default)]
pub struct Settings<Flags> {
    /// The identifier of the application.
    ///
    /// If provided, this identifier may be used to identify the application or
    /// communicate with it through the windowing system.
    pub id: Option<String>,

    /// The [`Window`] settings.
    pub window: window::Settings,

    /// The data needed to initialize an [`Application`].
    ///
    /// [`Application`]: crate::Application
    pub flags: Flags,
}

/// Converts the window settings into a `WindowBuilder` from `winit`.
pub fn window_builder(
    settings: window::Settings,
    title: &str,
    monitor: Option<MonitorHandle>,
    _id: Option<String>,
) -> WindowBuilder {
    let mut window_builder = WindowBuilder::new();

    let (width, height) = settings.size;

    window_builder = window_builder
        .with_title(title)
        .with_inner_size(winit::dpi::LogicalSize { width, height })
        .with_resizable(settings.resizable)
        .with_decorations(settings.decorations)
        .with_transparent(settings.transparent)
        .with_window_icon(settings.icon.and_then(conversion::icon))
        .with_window_level(conversion::window_level(settings.level))
        .with_visible(settings.visible);

    if let Some(position) =
        conversion::position(monitor.as_ref(), settings.size, settings.position)
    {
        window_builder = window_builder.with_position(position);
    }

    if let Some((width, height)) = settings.min_size {
        window_builder = window_builder
            .with_min_inner_size(winit::dpi::LogicalSize { width, height });
    }

    if let Some((width, height)) = settings.max_size {
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
                .with_parent_window(settings.platform_specific.parent);
        }
        window_builder = window_builder
            .with_drag_and_drop(settings.platform_specific.drag_and_drop);
    }

    #[cfg(target_os = "macos")]
    {
        use winit::platform::macos::WindowBuilderExtMacOS;

        window_builder = window_builder
            .with_title_hidden(settings.platform_specific.title_hidden)
            .with_titlebar_transparent(
                settings.platform_specific.titlebar_transparent,
            )
            .with_fullsize_content_view(
                settings.platform_specific.fullsize_content_view,
            );
    }

    window_builder
}
