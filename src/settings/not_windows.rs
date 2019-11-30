#![cfg(not(target_os = "windows"))]
//! Platform specific settings for not Windows.

/// The platform specific window settings of an application.
#[cfg(not(target_os = "windows"))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct PlatformSpecific {}

#[cfg(not(target_os = "windows"))]
impl From<PlatformSpecific> for iced_winit::settings::PlatformSpecific {
    fn from(_: PlatformSpecific) -> iced_winit::settings::PlatformSpecific {
        iced_winit::settings::PlatformSpecific {}
    }
}
