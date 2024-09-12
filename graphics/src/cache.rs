//! Cache computations and efficiently reuse them.
use std::cell::RefCell;
use std::fmt;
use std::mem;
use std::sync::atomic::{self, AtomicU64};

/// A simple cache that stores generated values to avoid recomputation.
///
/// Keeps track of the last generated value after clearing.
pub struct Cache<T> {
    group: Group,
    state: RefCell<State<T>>,
}

impl<T> Cache<T> {
    /// Creates a new empty [`Cache`].
    pub fn new() -> Self {
        Cache {
            group: Group::singleton(),
            state: RefCell::new(State::Empty { previous: None }),
        }
    }

    /// Creates a new empty [`Cache`] with the given [`Group`].
    ///
    /// Caches within the same group may reuse internal rendering storage.
    ///
    /// You should generally group caches that are likely to change
    /// together.
    pub fn with_group(group: Group) -> Self {
        assert!(
            !group.is_singleton(),
            "The group {group:?} cannot be shared!"
        );

        Cache {
            group,
            state: RefCell::new(State::Empty { previous: None }),
        }
    }

    /// Returns the [`Group`] of the [`Cache`].
    pub fn group(&self) -> Group {
        self.group
    }

    /// Puts the given value in the [`Cache`].
    ///
    /// Notice that, given this is a cache, a mutable reference is not
    /// necessary to call this method. You can safely update the cache in
    /// rendering code.
    pub fn put(&self, value: T) {
        *self.state.borrow_mut() = State::Filled { current: value };
    }

    /// Returns a reference cell to the internal [`State`] of the [`Cache`].
    pub fn state(&self) -> &RefCell<State<T>> {
        &self.state
    }

    /// Clears the [`Cache`].
    pub fn clear(&self) {
        let mut state = self.state.borrow_mut();

        let previous =
            mem::replace(&mut *state, State::Empty { previous: None });

        let previous = match previous {
            State::Empty { previous } => previous,
            State::Filled { current } => Some(current),
        };

        *state = State::Empty { previous };
    }
}

/// A cache group.
///
/// Caches that share the same group generally change together.
///
/// A cache group can be used to implement certain performance
/// optimizations during rendering, like batching or sharing atlases.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Group {
    id: u64,
    is_singleton: bool,
}

impl Group {
    /// Generates a new unique cache [`Group`].
    pub fn unique() -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(0);

        Self {
            id: NEXT.fetch_add(1, atomic::Ordering::Relaxed),
            is_singleton: false,
        }
    }

    /// Returns `true` if the [`Group`] can only ever have a
    /// single [`Cache`] in it.
    ///
    /// This is the default kind of [`Group`] assigned when using
    /// [`Cache::new`].
    ///
    /// Knowing that a [`Group`] will never be shared may be
    /// useful for rendering backends to perform additional
    /// optimizations.
    pub fn is_singleton(self) -> bool {
        self.is_singleton
    }

    fn singleton() -> Self {
        Self {
            is_singleton: true,
            ..Self::unique()
        }
    }
}

impl<T> fmt::Debug for Cache<T>
where
    T: fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use std::ops::Deref;

        let state = self.state.borrow();

        match state.deref() {
            State::Empty { previous } => {
                write!(f, "Cache::Empty {{ previous: {previous:?} }}")
            }
            State::Filled { current } => {
                write!(f, "Cache::Filled {{ current: {current:?} }}")
            }
        }
    }
}

impl<T> Default for Cache<T> {
    fn default() -> Self {
        Self::new()
    }
}

/// The state of a [`Cache`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum State<T> {
    /// The [`Cache`] is empty.
    Empty {
        /// The previous value of the [`Cache`].
        previous: Option<T>,
    },
    /// The [`Cache`] is filled.
    Filled {
        /// The current value of the [`Cache`]
        current: T,
    },
}

/// A piece of data that can be cached.
pub trait Cached: Sized {
    /// The type of cache produced.
    type Cache: Clone;

    /// Loads the [`Cache`] into a proper instance.
    ///
    /// [`Cache`]: Self::Cache
    fn load(cache: &Self::Cache) -> Self;

    /// Caches this value, producing its corresponding [`Cache`].
    ///
    /// [`Cache`]: Self::Cache
    fn cache(self, group: Group, previous: Option<Self::Cache>) -> Self::Cache;
}

#[cfg(debug_assertions)]
impl Cached for () {
    type Cache = ();

    fn load(_cache: &Self::Cache) -> Self {}

    fn cache(
        self,
        _group: Group,
        _previous: Option<Self::Cache>,
    ) -> Self::Cache {
    }
}
