//! Platform specific settings for Windows.
use raw_window_handle::RawWindowHandle;

/// The platform specific window settings of an application.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlatformSpecific {
    /// Parent window
    pub parent: Option<RawWindowHandle>,

    /// Drag and drop support
    pub drag_and_drop: bool,
}

impl Default for PlatformSpecific {
    fn default() -> Self {
        Self {
            parent: None,
            drag_and_drop: true,
        }
    }
}
