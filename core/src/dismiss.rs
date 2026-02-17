//! Dismiss events from the compositor.
//!
//! This module provides events for the dismiss protocol, notifying
//! the client when the user clicks/touches outside an armed dismiss group.

/// Dismiss events sent to layer shell surfaces.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Event {
    /// The user clicked/touched outside the armed dismiss group.
    /// The surface should close or hide itself.
    Requested,
}
