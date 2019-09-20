#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum Length {
    Fill,
    Shrink,
    Units(u16),
}
