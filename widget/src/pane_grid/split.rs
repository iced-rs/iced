/// A divider that splits a region in a [`PaneGrid`] into two different panes.
///
/// [`PaneGrid`]: super::PaneGrid
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Split(pub(super) usize);
