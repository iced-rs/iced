#![cfg(target_os = "windows")]
//! Platform specific settings for Windows.

/// The platform specific window settings of an application.
#[cfg(target_os = "windows")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct PlatformSpecific {
    /// Parent Window
    pub parent: Option<winapi::shared::windef::HWND>,
}

#[cfg(target_os = "windows")]
impl From<PlatformSpecific> for iced_winit::settings::PlatformSpecific {
    fn from(
        platform_specific: PlatformSpecific,
    ) -> iced_winit::settings::PlatformSpecific {
        iced_winit::settings::PlatformSpecific {
            parent: platform_specific.parent,
        }
    }
}
