use crate::pane_grid::Axis;

/// The arrangement of a [`PaneGrid`].
///
/// [`PaneGrid`]: struct.PaneGrid.html
#[derive(Debug, Clone)]
pub enum Configuration<T> {
    /// A split of the available space.
    Split {
        /// The direction of the split.
        axis: Axis,

        /// The ratio of the split in [0.0, 1.0].
        ratio: f32,

        /// The left/top [`Content`] of the split.
        ///
        /// [`Configuration`]: enum.Node.html
        a: Box<Configuration<T>>,

        /// The right/bottom [`Content`] of the split.
        ///
        /// [`Configuration`]: enum.Node.html
        b: Box<Configuration<T>>,
    },
    /// A [`Pane`].
    ///
    /// [`Pane`]: struct.Pane.html
    Pane(T),
}
