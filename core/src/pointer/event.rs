use crate::Point;
use crate::pointer;
use crate::pointer::{button, mouse, tablet};

/// A pointer event.
#[derive(Debug, Clone, PartialEq)]
pub enum Event {
    /// A pointer entered the window.
    PointerEntered {
        /// The position of the pointer when it entered the window.
        position: Point,

        /// The kind of pointer that entered the window.
        kind: pointer::Kind,
    },

    /// A pointer left the window.
    PointerLeft {
        /// The kind of pointer that left the window.
        kind: pointer::Kind,
    },

    /// A pointer moved within the window.
    PointerMoved {
        /// The new position of the pointer.
        position: Point,

        /// The source of the pointer movement.
        source: pointer::Source,
    },

    /// A pointer button was pressed.
    PointerPressed {
        /// The position of the pointer when the button was pressed.
        position: Point,

        /// The source and button that were pressed.
        button: button::Source,
    },

    /// A pointer button was released.
    PointerReleased {
        /// The position of the pointer when the button was released.
        position: Point,

        /// The source and button that were released.
        button: button::Source,
    },

    /// A mouse wheel or touchpad was scrolled.
    WheelScrolled {
        /// The scroll movement.
        delta: mouse::ScrollDelta,
    },
}

impl Event {
    /// Returns whether the event is a mouse left click, finger press, or tablet contact.
    pub fn is_primary_click(&self) -> bool {
        matches!(
            self,
            Self::PointerPressed {
                button: button::Source::Mouse(mouse::Button::Left)
                    | button::Source::Touch { .. }
                    | button::Source::TabletTool {
                        button: tablet::Button::Contact,
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
                button: button::Source::Mouse(mouse::Button::Left)
                    | button::Source::Touch { .. }
                    | button::Source::TabletTool {
                        button: tablet::Button::Contact,
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
                button: button::Source::Mouse(mouse::Button::Right)
                    | button::Source::TabletTool {
                        button: tablet::Button::Barrel,
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
                button: button::Source::Mouse(mouse::Button::Right)
                    | button::Source::TabletTool {
                        button: tablet::Button::Barrel,
                        ..
                    },
                ..
            }
        )
    }
}
