//! Build touch events.
use crate::{Point, PointerInput};

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

/// The touch input state.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum Touch {
    /// The touch has a defined position.
    Available(Point),

    /// The touch is currently unavailable (i.e. is not registered by the input device).
    #[default]
    Unavailable,
}

impl PointerInput for Touch {
    /// Returns the absolute position of the [`Touch`], if available.
    fn position(self) -> Option<Point> {
        match self {
            Touch::Available(position) => Some(position),
            Touch::Unavailable => None,
        }
    }
}
