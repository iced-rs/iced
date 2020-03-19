/// A divider that splits a region in a [`PaneGrid`] into two different panes.
///
/// [`PaneGrid`]: struct.PaneGrid.html
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Split(pub(super) usize);
