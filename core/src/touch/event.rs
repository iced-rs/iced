use super::{Finger, Phase};
use crate::Point;

/// A touch event.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Event {
    /// The finger of the touch.
    pub finger: Finger,

    /// The position of the touch.
    pub position: Point,

    /// The state of the touch.
    pub phase: Phase,
}
