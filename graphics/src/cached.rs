use crate::Primitive;

use std::sync::Arc;

/// A piece of data that can be cached.
pub trait Cached: Sized {
    /// The type of cache produced.
    type Cache;

    /// Loads the [`Cache`] into a proper instance.
    ///
    /// [`Cache`]: Self::Cache
    fn load(cache: &Self::Cache) -> Self;

    /// Caches this value, producing its corresponding [`Cache`].
    ///
    /// [`Cache`]: Self::Cache
    fn cache(self) -> Self::Cache;
}

impl<T> Cached for Primitive<T> {
    type Cache = Arc<Self>;

    fn load(cache: &Arc<Self>) -> Self {
        Self::Cache {
            content: cache.clone(),
        }
    }

    fn cache(self) -> Arc<Self> {
        Arc::new(self)
    }
}
