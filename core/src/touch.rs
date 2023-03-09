//! Build touch events.
use crate::Point;

/// A touch interaction.
#[derive(Debug, Clone, Copy, PartialEq)]
#[allow(missing_docs)]
pub enum Event {
    /// A touch interaction was started.
    FingerPressed { id: Finger, position: Point },

    /// An on-going touch interaction was moved.
    FingerMoved { id: Finger, position: Point },

    /// A touch interaction was ended.
    FingerLifted { id: Finger, position: Point },

    /// A touch interaction was canceled.
    FingerLost { id: Finger, position: Point },
}

/// A unique identifier representing a finger on a touch interaction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Finger(pub u64);
