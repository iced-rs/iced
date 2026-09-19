/// A gamepad button.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Button {
    /// The south button on the action pad.
    South,

    /// The east button on the action pad.
    East,

    /// The north button on the action pad.
    North,

    /// The west button on the action pad.
    West,

    /// The left shoulder. Commonly known as `LB`, `L1` or `L`.
    LeftShoulder,

    /// The left trigger. Commonly known as `LT`, `L2` or `ZL`.
    LeftTrigger,

    /// The right shoulder. Commonly known as `RB`, `R1` or `R`.
    RightShoulder,

    /// The right trigger. Commonly known as `RT`, `R2` or `ZR`.
    RightTrigger,

    /// The left thumbstick button. Commonly known as `LS` or `L3`.
    LeftThumb,

    /// The right thumbstick button. Commonly known as `RS` or `R3`.
    RightThumb,

    /// The `up` directional pad button.
    DPadUp,

    /// The `down` directional pad button.
    DPadDown,

    /// The `left` directional pad button.
    DPadLeft,

    /// The `right` directional pad button.
    DPadRight,

    /// The `select` button in the menu pad.
    Select,

    /// The `start` button in the menu pad.
    Start,

    /// An unknown gamepad button.
    Unknown,
}
