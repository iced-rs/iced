//! Platform specific settings for Windows.

/// The platform specific window settings of an application.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlatformSpecific {
    /// Drag and drop support
    pub drag_and_drop: bool,

    /// Whether show or hide the window icon in the taskbar.
    pub skip_taskbar: bool,
}

impl Default for PlatformSpecific {
    fn default() -> Self {
        Self {
            drag_and_drop: true,
            skip_taskbar: false,
        }
    }
}
