use std::hash::Hash;

use std::sync::atomic::{self, AtomicU64};

/// The id of the window.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Id(u64);

static COUNT: AtomicU64 = AtomicU64::new(1);

impl Id {
    /// Creates a new unique window [`Id`].
    pub fn unique() -> Id {
        Id(COUNT.fetch_add(1, atomic::Ordering::Relaxed))
    }
}
