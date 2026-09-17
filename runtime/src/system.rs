//! Access the native system.
use crate::core::Color;
use crate::core::theme;
use crate::futures::futures::channel::oneshot;
use crate::futures::subscription::{self, Subscription};
use crate::task::{self, Task};

/// An operation to be performed on the system.
#[derive(Debug)]
pub enum Action {
    /// Send available system information.
    GetInformation(oneshot::Sender<Information>),

    /// Send the current system theme mode.
    GetTheme(oneshot::Sender<theme::Mode>),

    /// Notify to the runtime that the system theme has changed.
    NotifyTheme(theme::Mode),

    /// Send the current system accent color.
    GetAccentColor(oneshot::Sender<Option<Color>>),

    /// Notify to the runtime that the system accent color has changed.
    NotifyAccentColor(Option<Color>),
}

/// Contains information about the system (e.g. system name, processor, memory, graphics adapter).
#[derive(Clone, Debug)]
pub struct Information {
    /// The operating system name
    pub system_name: Option<String>,
    /// Operating system kernel version
    pub system_kernel: Option<String>,
    /// Long operating system version
    ///
    /// Examples:
    /// - MacOS 10.15 Catalina
    /// - Windows 10 Pro
    /// - Ubuntu 20.04 LTS (Focal Fossa)
    pub system_version: Option<String>,
    /// Short operating system version number
    pub system_short_version: Option<String>,
    /// Detailed processor model information
    pub cpu_brand: String,
    /// The number of physical cores on the processor
    pub cpu_cores: Option<usize>,
    /// Total RAM size, in bytes
    pub memory_total: u64,
    /// Memory used by this process, in bytes
    pub memory_used: Option<u64>,
    /// Underlying graphics backend for rendering
    pub graphics_backend: String,
    /// Model information for the active graphics adapter
    pub graphics_adapter: String,
}

/// Returns available system information.
pub fn information() -> Task<Information> {
    task::oneshot(|channel| crate::Action::System(Action::GetInformation(channel)))
}

/// Returns the current system theme.
pub fn theme() -> Task<theme::Mode> {
    task::oneshot(|sender| crate::Action::System(Action::GetTheme(sender)))
}

/// Subscribes to system theme changes.
pub fn theme_changes() -> Subscription<theme::Mode> {
    #[derive(Hash)]
    struct ThemeChanges;

    subscription::filter_map(ThemeChanges, |event| {
        let subscription::Event::SystemThemeChanged(mode) = event else {
            return None;
        };

        Some(mode)
    })
}

/// Returns the current system accent color.
///
/// The [`Task`] produces `None` when the system has no accent color
/// preference or when accent color detection is unsupported on the
/// current platform.
///
/// Accent color detection is currently only available on Linux with
/// the `linux-theme-detection` feature enabled.
pub fn accent_color() -> Task<Option<Color>> {
    task::oneshot(|sender| crate::Action::System(Action::GetAccentColor(sender)))
}

/// Subscribes to system accent color changes.
///
/// `None` means the system no longer has an accent color preference.
pub fn accent_color_changes() -> Subscription<Option<Color>> {
    #[derive(Hash)]
    struct AccentColorChanges;

    subscription::filter_map(AccentColorChanges, |event| {
        let subscription::Event::SystemAccentColorChanged(color) = event else {
            return None;
        };

        Some(color)
    })
}
