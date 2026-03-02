//! Platform-specific actions.
//!
//! This module provides actions for platform-specific features that
//! aren't available through the standard iced API.

use std::fmt;

/// Wayland-specific actions
#[cfg(all(
    unix,
    not(any(target_os = "macos", target_os = "ios", target_os = "android"))
))]
pub mod wayland;

/// Platform-specific action.
#[derive(Clone)]
pub enum Action {
    /// Wayland-specific action
    #[cfg(all(
        unix,
        not(any(target_os = "macos", target_os = "ios", target_os = "android"))
    ))]
    Wayland(wayland::Action),
}

impl fmt::Debug for Action {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            #[cfg(all(
                unix,
                not(any(target_os = "macos", target_os = "ios", target_os = "android"))
            ))]
            Action::Wayland(action) => f.debug_tuple("Wayland").field(action).finish(),
            #[cfg(not(all(
                unix,
                not(any(target_os = "macos", target_os = "ios", target_os = "android"))
            )))]
            _ => Ok(()),
        }
    }
}
