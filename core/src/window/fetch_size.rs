/// A `struct` defining which size to fetch.
#[derive(Debug, Clone, Copy)]
pub enum SizeType {
    /// Inner size. (not including title bars and so other OS decorations)
    Inner,
    /// Outer size. (including everything)
    Outer,
}