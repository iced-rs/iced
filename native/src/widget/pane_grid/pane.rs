/// A rectangular region in a [`PaneGrid`] used to display widgets.
///
/// [`PaneGrid`]: crate::widget::PaneGrid
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Pane(pub(super) usize);
