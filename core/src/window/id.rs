use std::collections::hash_map::DefaultHasher;
use std::fmt::{Display, Formatter};
use std::hash::{Hash, Hasher};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
/// The ID of the window.
///
/// Internally Iced uses `window::Id::MAIN` as the first window spawned.
pub struct Id(u64);

impl Id {
    /// The reserved window ID for the primary window in an Iced application.
    pub const MAIN: Self = Id(0);

    /// Creates a new unique window ID.
    pub fn new(id: impl Hash) -> Id {
        let mut hasher = DefaultHasher::new();
        id.hash(&mut hasher);

        Id(hasher.finish())
    }
}

impl Display for Id {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Id({})", self.0)
    }
}
