//! Surface visibility events from the layer-surface-visibility protocol.
//!
//! These events fire when a surface is hidden or shown via the
//! `zcosmic_layer_surface_visibility_v1` protocol, letting the application
//! throttle work (animations, subscriptions) while invisible.

/// Surface visibility events.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Event {
    /// The surface is now visible.
    Shown,
    /// The surface is now hidden by the compositor.
    Hidden,
}
