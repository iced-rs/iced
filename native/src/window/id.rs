use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
/// TODO(derezzedex)
pub struct Id(u64);

impl Id {
    /// TODO(derezzedex): maybe change `u64` to an enum `Type::{Single, Multi(u64)}`
    pub const MAIN: Self = Id(0);

    /// TODO(derezzedex)
    pub fn new(id: impl Hash) -> Id {
        let mut hasher = DefaultHasher::new();
        id.hash(&mut hasher);

        Id(hasher.finish())
    }
}
