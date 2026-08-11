use crate::Point;

use super::{ButtonSource, MouseButton, PointerKind, PointerSource, TabletToolButton};

/// A pointer event.
#[derive(Debug, Clone, PartialEq)]
pub enum Event {
    /// A pointer entered the window.
    PointerEntered {
        /// The position of the pointer when it entered the window.
        position: Point,

        /// The kind of pointer that entered the window.
        kind: PointerKind,
    },

    /// A pointer left the window.
    PointerLeft {
        /// The kind of pointer that left the window.
        kind: PointerKind,
    },

    /// A pointer moved within the window.
    PointerMoved {
        /// The new position of the pointer.
        position: Point,

        /// The source of the pointer movement.
        source: PointerSource,
    },

    /// A pointer button was pressed.
    PointerPressed {
        /// The position of the pointer when the button was pressed.
        position: Point,

        /// The source and button that were pressed.
        button: ButtonSource,
    },

    /// A pointer button was released.
    PointerReleased {
        /// The position of the pointer when the button was released.
        position: Point,

        /// The source and button that were released.
        button: ButtonSource,
    },

    /// A mouse wheel or touchpad was scrolled.
    MouseWheel {
        /// The scroll movement.
        delta: ScrollDelta,
    },
}

impl Event {
    /// Returns whether the event is a mouse left click, finger press, or tablet contact.
    pub fn is_primary_click(&self) -> bool {
        matches!(
            self,
            Self::PointerPressed {
                button: ButtonSource::Mouse(MouseButton::Left)
                    | ButtonSource::Touch { .. }
                    | ButtonSource::TabletTool {
                        button: TabletToolButton::Contact,
                        ..
                    },
                ..
            }
        )
    }

    /// Returns whether the event is a mouse left release, finger lift, or tablet contact release.
    pub fn is_primary_release(&self) -> bool {
        matches!(
            self,
            Self::PointerReleased {
                button: ButtonSource::Mouse(MouseButton::Left)
                    | ButtonSource::Touch { .. }
                    | ButtonSource::TabletTool {
                        button: TabletToolButton::Contact,
                        ..
                    },
                ..
            }
        )
    }

    /// Returns whether the event is a mouse right click or tablet barrel press.
    pub fn is_secondary_click(&self) -> bool {
        matches!(
            self,
            Self::PointerPressed {
                button: ButtonSource::Mouse(MouseButton::Right)
                    | ButtonSource::TabletTool {
                        button: TabletToolButton::Barrel,
                        ..
                    },
                ..
            }
        )
    }

    /// Returns whether the event is a mouse right release or tablet barrel lift.
    pub fn is_secondary_release(&self) -> bool {
        matches!(
            self,
            Self::PointerReleased {
                button: ButtonSource::Mouse(MouseButton::Right)
                    | ButtonSource::TabletTool {
                        button: TabletToolButton::Barrel,
                        ..
                    },
                ..
            }
        )
    }
}

/// A scroll movement.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ScrollDelta {
    /// A line-based scroll movement.
    Lines {
        /// The number of horizontal lines scrolled.
        x: f32,

        /// The number of vertical lines scrolled.
        y: f32,
    },

    /// A pixel-based scroll movement.
    Pixels {
        /// The number of horizontal pixels scrolled.
        x: f32,

        /// The number of vertical pixels scrolled.
        y: f32,
    },
}
