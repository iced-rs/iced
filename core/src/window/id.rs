use std::hash::Hash;

use std::sync::atomic::{self, AtomicU64};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
/// The id of the window.
///
/// Internally Iced reserves `window::Id::MAIN` for the first window spawned.
pub struct Id(u64);

static COUNT: AtomicU64 = AtomicU64::new(1);

impl Id {
    /// The reserved window [`Id`] for the first window in an Iced application.
    pub const MAIN: Self = Id(0);

    /// Creates a new unique window [`Id`].
    pub fn unique() -> Id {
        Id(COUNT.fetch_add(1, atomic::Ordering::Relaxed))
    }
}
