//! Handle pointer buttons.

use crate::pointer::{mouse, tablet, touch};

/// The source and button of a pointer button event.
#[derive(Debug, Clone, PartialEq)]
pub enum Source {
    /// A mouse button.
    Mouse(mouse::Button),

    /// A touch contact.
    Touch {
        /// The identifier of the finger.
        finger_id: touch::Finger,

        /// The force of the touch, if reported by the device.
        force: Option<touch::Force>,
    },

    /// A tablet tool button.
    TabletTool {
        /// The kind of tablet tool.
        kind: tablet::Kind,

        /// The button that changed state.
        button: tablet::Button,

        /// Data describing how the tool is held and used.
        data: tablet::Data,
    },

    /// A button from an unknown pointer source.
    Unknown(u16),
}
