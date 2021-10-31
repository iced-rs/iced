use crate::widget::pane_grid::Axis;

/// The arrangement of a [`PaneGrid`].
///
/// [`PaneGrid`]: crate::pane_grid::PaneGrid
#[derive(Debug, Clone)]
pub enum Configuration<T> {
    /// A split of the available space.
    Split {
        /// The direction of the split.
        axis: Axis,

        /// The ratio of the split in [0.0, 1.0].
        ratio: f32,

        /// The left/top [`Configuration`] of the split.
        a: Box<Configuration<T>>,

        /// The right/bottom [`Configuration`] of the split.
        b: Box<Configuration<T>>,
    },
    /// A [`Pane`].
    ///
    /// [`Pane`]: crate::pane_grid::Pane
    Pane(T),
}
