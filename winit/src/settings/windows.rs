#![cfg(target_os = "windows")]
//! Platform specific settings for Windows.

/// The platform specific window settings of an application.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct PlatformSpecific {
    /// Parent Window
    pub parent: Option<winapi::shared::windef::HWND>,
}
