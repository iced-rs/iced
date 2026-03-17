//! Handle gesture events

/// A gesture event.
///
/// _**Note:** This type is largely incomplete! If you need to track
/// additional events, feel free to [open an issue] and share your use case!_
///
/// [open an issue]: https://github.com/iced-rs/iced/issues
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Event {
    /// N-finger pan gesture
    /// Only available on **Wayland**.
    Pan {
        /// Change in pixels of pan gesture from last update.
        delta: Delta,
        /// Describes touch-screen input state.
        phase: Phase,
    },
    /// Two-finger pinch gesture, often used for magnification.
    /// Only available on **macOS**, and **Wayland**.
    Pinch {
        /// Pinch delta. Positive values indicate magnification (zooming in).
        delta: f64,
        /// Describes touch-screen input state.
        phase: Phase,
    },
    /// Two-finger rotation gesture.
    /// Only available on **macOS** and **Wayland**.
    Rotate {
        /// Rotation delta. Positive delta values indicate rotation counterclockwise.
        delta: f32,
        /// Describes touch-screen input state.
        phase: Phase,
    },
    /// Double tap gesture.
    /// Only available on **macOS 10.8** and later.
    DoubleTap,
}

/// Change in pixels of pan gesture from last update.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Delta {
    /// Change in pixels of pan gesture in X axis from last update.
    pub x: f32,
    /// Change in pixels of pan gesture in Y axis from last update.
    pub y: f32,
}

/// Describes gesture input state.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Phase {
    /// Started
    Started,
    /// Moded
    Moved,
    /// Ended
    Ended,
    /// Cancelled
    Cancelled,
}
