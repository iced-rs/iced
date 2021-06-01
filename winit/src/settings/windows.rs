#![cfg(target_os = "windows")]
//! Platform specific settings for Windows.

/// The platform specific window settings of an application.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlatformSpecific {
    /// Parent window
    pub parent: Option<winapi::shared::windef::HWND>,

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
