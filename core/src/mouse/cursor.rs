/// The cursor state of the [`Mouse`](crate::Mouse).
#[derive(Debug, Eq, PartialEq, Clone, Copy, PartialOrd, Ord, Default)]
#[allow(missing_docs)]
pub enum Cursor {
    #[default]
    Undefined,
    Idle,
    Pointer,
    Grab,
    Text,
    Crosshair,
    Working,
    Grabbing,
    ResizingHorizontally,
    ResizingVertically,
    ResizingDiagonallyUp,
    ResizingDiagonallyDown,
    NotAllowed,
    ZoomIn,
    ZoomOut,
    Cell,
    Move,
    Copy,
    Help,
}
