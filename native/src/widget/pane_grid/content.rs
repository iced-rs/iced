use crate::pane_grid::Axis;

/// The content of a [`PaneGrid`].
///
/// [`PaneGrid`]: struct.PaneGrid.html
#[derive(Debug, Clone)]
pub enum Content<T> {
    /// A split of the available space.
    Split {
        /// The direction of the split.
        axis: Axis,

        /// The ratio of the split in [0.0, 1.0].
        ratio: f32,

        /// The left/top [`Content`] of the split.
        ///
        /// [`Content`]: enum.Node.html
        a: Box<Content<T>>,

        /// The right/bottom [`Content`] of the split.
        ///
        /// [`Content`]: enum.Node.html
        b: Box<Content<T>>,
    },
    /// A [`Pane`].
    ///
    /// [`Pane`]: struct.Pane.html
    Pane(T),
}
