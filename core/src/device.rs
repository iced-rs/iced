//! Handle device events.
//!
//! Device events are raw hardware events that are not associated
//! with any particular window.
//!
//! # Performance Warning
//! Device events fire at very high frequency (1000+ events/sec for mouse).
//! Always filter events in your callback - do NOT produce a message for every event!
use crate::keyboard;

/// A raw device event.
///
/// Device events are global hardware events not tied to any window.
/// These include raw mouse motion, button presses, and key events
/// that bypass the OS input processing.
///
/// # Warning
/// These events fire at very high frequency! Always filter in your
/// subscription callback and return `None` for events you don't need.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Event {
    /// A device has been added to the system.
    Added,

    /// A device has been removed from the system.
    Removed,

    /// Raw mouse motion delta (unaccelerated, not affected by OS settings).
    ///
    /// This is useful for FPS games, 3D editors, or any application
    /// that needs raw mouse input.
    ///
    /// **Warning:** Fires at very high frequency!
    MouseMotion {
        /// The (x, y) delta in device units.
        delta: (f64, f64),
    },

    /// Raw mouse wheel scroll event.
    MouseWheel {
        /// The scroll delta.
        delta: MouseScrollDelta,
    },

    /// Raw axis motion from a device (e.g., joystick, gamepad).
    Motion {
        /// The axis identifier.
        axis: u32,
        /// The axis value.
        value: f64,
    },

    /// A raw button press or release.
    ///
    /// The button ID is device-specific and may not correspond
    /// to standard mouse buttons.
    Button {
        /// The device-specific button identifier.
        button: u32,
        /// Whether the button was pressed (`true`) or released (`false`).
        pressed: bool,
    },

    /// A raw key press or release.
    ///
    /// This provides physical key events that bypass OS keyboard layout
    /// processing. Useful for games that need consistent key positions.
    Key {
        /// The physical key that was pressed or released.
        physical_key: keyboard::key::Physical,
        /// Whether the key was pressed (`true`) or released (`false`).
        pressed: bool,
    },
}

/// Controls when device events are delivered to the application.
///
/// This corresponds to winit's `DeviceEvents` setting.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Filter {
    /// Device events are always delivered, regardless of window focus.
    Always,
    /// Device events are never delivered.
    Never,
    /// Device events are only delivered when a window has focus.
    ///
    /// This is the default behavior.
    #[default]
    WhenFocused,
}

/// Raw mouse scroll delta from a device event.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MouseScrollDelta {
    /// Scroll amount in lines.
    Lines {
        /// Horizontal scroll amount.
        x: f32,
        /// Vertical scroll amount.
        y: f32,
    },
    /// Scroll amount in pixels.
    Pixels {
        /// Horizontal scroll amount.
        x: f32,
        /// Vertical scroll amount.
        y: f32,
    },
}
