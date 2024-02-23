//! Platform specific settings for Windows.
use raw_window_handle::RawWindowHandle;

/// The platform specific window settings of an application.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlatformSpecific {
    /// Parent window
    pub parent: Option<RawWindowHandle>,

    /// Drag and drop support
    pub drag_and_drop: bool,

    /// Whether show or hide the window icon in the taskbar.
    pub skip_taskbar: bool,

    /// Shows or hides the background drop shadow for undecorated windows.
    ///
    /// The shadow is hidden by default.
    /// Enabling the shadow causes a thin 1px line to appear on the top of the window.
    pub undecorated_shadow: bool,
}

impl Default for PlatformSpecific {
    fn default() -> Self {
        Self {
            parent: None,
            drag_and_drop: true,
            skip_taskbar: false,
            undecorated_shadow: false,
        }
    }
}
