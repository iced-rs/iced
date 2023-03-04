/// A divider that splits a region in a [`PaneGrid`] into two different panes.
///
/// [`PaneGrid`]: crate::widget::PaneGrid
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Split(pub(super) usize);
