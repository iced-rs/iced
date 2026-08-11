//! Build touch events.

use crate::Point;

pub use crate::pointer::touch::Finger;

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
