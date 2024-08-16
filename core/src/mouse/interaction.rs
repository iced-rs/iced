/// The interaction of a mouse cursor.
#[derive(Debug, Eq, PartialEq, Clone, Copy, PartialOrd, Ord, Default)]
#[allow(missing_docs)]
pub enum Interaction {
    #[default]
    None,
    Idle,
    Pointer,
    Grab,
    Text,
    Crosshair,
    Working,
    Grabbing,
    ResizingHorizontally,
    ResizingVertically,
    ResizingDiagonalUp,
    ResizingDiagonalDown,
    NotAllowed,
    ZoomIn,
    ZoomOut,
    Cell,
    Move,
}
