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
    fn cache(self, previous: Option<Self::Cache>) -> Self::Cache;
}

#[cfg(debug_assertions)]
impl Cached for () {
    type Cache = ();

    fn load(_cache: &Self::Cache) -> Self {}

    fn cache(self, _previous: Option<Self::Cache>) -> Self::Cache {}
}
