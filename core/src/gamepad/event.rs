use crate::gamepad::{Axis, Button, Id, Thumbstick};

/// A gamepad event.
#[derive(Debug, Clone, PartialEq)]
pub enum Event {
    /// A gamepad has been connected.
    Connected(Id),

    /// A gamepad has been disconnected.
    Disconnected(Id),

    /// A gamepad button has been pressed.
    ButtonPressed {
        /// The [`Id`] of the gamepad.
        id: Id,

        /// The gamepad [`Button`] pressed.
        button: Button,

        /// Wether the button is being held.
        repeat: bool,
    },

    /// The value of a button has changed.
    ButtonChanged {
        /// The [`Id`] of the gamepad.
        id: Id,

        /// The gamepad [`Button`] pressed.
        button: Button,

        /// The value of the [`Button`].
        ///
        /// Value is between `0.0` and `1.0`.
        value: f32,
    },

    /// A gamepad button has been released.
    ButtonReleased {
        /// The [`Id`] of the gamepad.
        id: Id,

        /// The [`Button`] of the gamepad.
        button: Button
    },

    /// A gamepad thumbstick axis value changed.
    AxisChanged {
        /// The [`Id`] of the gamepad.
        id: Id,

        /// The [`Thumbstick`] that was used.
        thumbstick: Thumbstick,

        /// The [`Axis`] that has changed.
        axis: Axis,

        /// The current value of the axis. Value is between `-1.0` and `1.0`.
        value: f32,
    },
}
