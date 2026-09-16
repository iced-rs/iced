//! Build touch events.
use crate::{Point, PointerInput, Rectangle};

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
    Unavailable(Point),

    /// The state is unknown
    #[default]
    Unknown,
}

impl PointerInput for Touch {
    /// Returns the absolute position of the [`Touch`], if available.
    fn position(self) -> Option<Point> {
        match self {
            Touch::Available(position) => Some(position),
            _ => None,
        }
    }
}

impl Touch {
    /// Returns the last observable absolute position of the [`Touch`]
    pub fn last_position(self) -> Option<Point> {
        match self {
            Touch::Available(position) | Touch::Unavailable(position) => Some(position),
            _ => None,
        }
    }

    /// Returns the last absolute position of the [`PointerInput`], if available and inside
    /// the given bounds.
    ///
    /// If the [`PointerInput`] has not been over the provided bounds, this method will
    /// return `None`.
    pub fn last_position_over(self, bounds: Rectangle) -> Option<Point> {
        self.last_position().filter(|p| bounds.contains(*p))
    }

    /// Returns true if the [`PointerInput`] has been over the given `bounds`.
    pub fn has_been_over(self, bounds: Rectangle) -> bool {
        self.last_position_over(bounds).is_some()
    }
}
