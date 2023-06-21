use crate::core::{Layout, Point};

/// A pane that can be dragged.
pub trait Draggable {
    /// Returns whether the [`Draggable`] with the given [`Layout`] can be picked
    /// at the provided cursor position.
    fn can_be_dragged_at(&self, layout: Layout<'_>, cursor: Point) -> bool;
}
