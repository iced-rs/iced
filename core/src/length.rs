/// The strategy used to fill space in a specific dimension.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum Length {
    Fill,
    Shrink,
    Units(u16),
}
