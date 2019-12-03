#![cfg(not(target_os = "windows"))]
//! Platform specific settings for not Windows.

/// The platform specific window settings of an application.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct PlatformSpecific {}
