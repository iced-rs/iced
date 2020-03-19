/// A rectangular region in a [`PaneGrid`] used to display widgets.
///
/// [`PaneGrid`]: struct.PaneGrid.html
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Pane(pub(super) usize);
