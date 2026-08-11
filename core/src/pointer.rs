//! Handle pointer events.

pub mod button;
pub mod mouse;
pub mod tablet;
pub mod touch;

mod event;

pub use event::Event;

/// The kind of device that produced a pointer event.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Kind {
    /// A mouse.
    Mouse,

    /// A finger touching a screen.
    Touch(touch::Finger),

    /// A tablet tool.
    TabletTool(tablet::Kind),

    /// An unknown pointer device.
    Unknown,
}

/// The device and associated data that produced a pointer event.
#[derive(Debug, Clone, PartialEq)]
pub enum Source {
    /// A mouse.
    Mouse,

    /// A finger touching a screen.
    Touch {
        /// The identifier of the finger.
        finger_id: touch::Finger,

        /// The force of the touch, if reported by the device.
        force: Option<touch::Force>,
    },

    /// A tablet tool.
    TabletTool {
        /// The kind of tablet tool.
        kind: tablet::Kind,

        /// Data describing how the tool is held and used.
        data: tablet::Data,
    },

    /// An unknown pointer device.
    Unknown,
}

impl From<Source> for crate::pointer::Kind {
    fn from(source: Source) -> Self {
        match source {
            Source::Mouse => Self::Mouse,
            Source::Touch { finger_id, .. } => Self::Touch(finger_id),
            Source::TabletTool { kind, .. } => Self::TabletTool(kind),
            Source::Unknown => Self::Unknown,
        }
    }
}
