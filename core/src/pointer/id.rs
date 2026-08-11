/// An identifier for a finger in a touch interaction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FingerId(usize);

impl FingerId {
    /// Converts the identifier into its underlying integer.
    pub const fn into_raw(self) -> usize {
        self.0
    }

    /// Constructs an identifier from its underlying integer.
    pub const fn from_raw(id: usize) -> Self {
        Self(id)
    }
}
