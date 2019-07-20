#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub enum Button {
    Left,
    Right,
    Middle,
    Other(u8),
}
