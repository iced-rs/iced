/// The interaction of a mouse cursor.
#[derive(Debug, Eq, PartialEq, Clone, Copy, PartialOrd, Ord)]
#[allow(missing_docs)]
pub enum Interaction {
    Idle,
    Pointer,
    Grab,
    Text,
    Crosshair,
    Working,
    Grabbing,
    ResizingHorizontally,
    ResizingVertically,
}

impl Default for Interaction {
    fn default() -> Interaction {
        Interaction::Idle
    }
}
