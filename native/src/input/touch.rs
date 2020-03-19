//! Build touch events.

use crate::Point;

/// A touch interaction.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Touch {
    /// The finger of the touch.
    pub finger: Finger,

    /// The position of the touch.
    pub position: Point,

    /// The state of the touch.
    pub phase: Phase,
}

/// A unique identifier representing a finger on a touch interaction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Finger(pub u64);

/// The state of a touch interaction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Phase {
    /// A touch interaction was started.
    Started,

    /// An on-going touch interaction was moved.
    Moved,

    /// A touch interaction was ended.
    Ended,

    /// A touch interaction was canceled.
    Canceled,
}
