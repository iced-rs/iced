use super::{FingerId, Force, TabletToolData, TabletToolKind};

/// The kind of device that produced a pointer event.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PointerKind {
    /// A mouse.
    Mouse,

    /// A finger touching a screen.
    Touch(FingerId),

    /// A tablet tool.
    TabletTool(TabletToolKind),

    /// An unknown pointer device.
    Unknown,
}

/// The device and associated data that produced a pointer event.
#[derive(Debug, Clone, PartialEq)]
pub enum PointerSource {
    /// A mouse.
    Mouse,

    /// A finger touching a screen.
    Touch {
        /// The identifier of the finger.
        finger_id: FingerId,

        /// The force of the touch, if reported by the device.
        force: Option<Force>,
    },

    /// A tablet tool.
    TabletTool {
        /// The kind of tablet tool.
        kind: TabletToolKind,

        /// Data describing how the tool is held and used.
        data: TabletToolData,
    },

    /// An unknown pointer device.
    Unknown,
}

impl From<PointerSource> for PointerKind {
    fn from(source: PointerSource) -> Self {
        match source {
            PointerSource::Mouse => Self::Mouse,
            PointerSource::Touch { finger_id, .. } => Self::Touch(finger_id),
            PointerSource::TabletTool { kind, .. } => Self::TabletTool(kind),
            PointerSource::Unknown => Self::Unknown,
        }
    }
}
