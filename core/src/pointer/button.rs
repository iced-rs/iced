use super::{FingerId, Force, TabletToolData, TabletToolKind};

/// The source and button of a pointer button event.
#[derive(Debug, Clone, PartialEq)]
pub enum ButtonSource {
    /// A mouse button.
    Mouse(MouseButton),

    /// A touch contact.
    Touch {
        /// The identifier of the finger.
        finger_id: FingerId,

        /// The force of the touch, if reported by the device.
        force: Option<Force>,
    },

    /// A tablet tool button.
    TabletTool {
        /// The kind of tablet tool.
        kind: TabletToolKind,

        /// The button that changed state.
        button: TabletToolButton,

        /// Data describing how the tool is held and used.
        data: TabletToolData,
    },

    /// A button from an unknown pointer source.
    Unknown(u16),
}

/// A button of a mouse.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
#[allow(missing_docs)]
pub enum MouseButton {
    Left = 0,
    Right = 1,
    Middle = 2,
    Back = 3,
    Forward = 4,
    Button6 = 5,
    Button7 = 6,
    Button8 = 7,
    Button9 = 8,
    Button10 = 9,
    Button11 = 10,
    Button12 = 11,
    Button13 = 12,
    Button14 = 13,
    Button15 = 14,
    Button16 = 15,
    Button17 = 16,
    Button18 = 17,
    Button19 = 18,
    Button20 = 19,
    Button21 = 20,
    Button22 = 21,
    Button23 = 22,
    Button24 = 23,
    Button25 = 24,
    Button26 = 25,
    Button27 = 26,
    Button28 = 27,
    Button29 = 28,
    Button30 = 29,
    Button31 = 30,
    Button32 = 31,
}

impl MouseButton {
    /// Constructs a mouse button from a value in the range `0..=31`.
    pub fn try_from_u8(button: u8) -> Option<Self> {
        Some(match button {
            0 => Self::Left,
            1 => Self::Right,
            2 => Self::Middle,
            3 => Self::Back,
            4 => Self::Forward,
            5 => Self::Button6,
            6 => Self::Button7,
            7 => Self::Button8,
            8 => Self::Button9,
            9 => Self::Button10,
            10 => Self::Button11,
            11 => Self::Button12,
            12 => Self::Button13,
            13 => Self::Button14,
            14 => Self::Button15,
            15 => Self::Button16,
            16 => Self::Button17,
            17 => Self::Button18,
            18 => Self::Button19,
            19 => Self::Button20,
            20 => Self::Button21,
            21 => Self::Button22,
            22 => Self::Button23,
            23 => Self::Button24,
            24 => Self::Button25,
            25 => Self::Button26,
            26 => Self::Button27,
            27 => Self::Button28,
            28 => Self::Button29,
            29 => Self::Button30,
            30 => Self::Button31,
            31 => Self::Button32,
            _ => return None,
        })
    }
}

/// A button of a tablet tool.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TabletToolButton {
    /// Contact between the tool and tablet surface.
    Contact,

    /// The tool's barrel button.
    Barrel,

    /// Another tablet tool button.
    Other(u16),
}

impl From<TabletToolButton> for Option<MouseButton> {
    fn from(button: TabletToolButton) -> Self {
        Some(match button {
            TabletToolButton::Contact => MouseButton::Left,
            TabletToolButton::Barrel => MouseButton::Right,
            TabletToolButton::Other(1) => MouseButton::Middle,
            TabletToolButton::Other(3) => MouseButton::Back,
            TabletToolButton::Other(4) => MouseButton::Forward,
            TabletToolButton::Other(_) => return None,
        })
    }
}
