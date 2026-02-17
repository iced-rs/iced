//! Auto-hide events from the compositor.
//!
//! This module provides events related to auto-hide layer shell surfaces,
//! notifying the client when the compositor changes visibility state.

/// Auto-hide events sent to layer shell surfaces.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Event {
    /// The surface is now fully visible (slide-in complete).
    Shown,
    /// The surface is now fully hidden (slide-out complete).
    Hidden,
}
