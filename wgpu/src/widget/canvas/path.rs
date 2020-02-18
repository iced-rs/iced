//! Build different kinds of 2D shapes.
pub mod arc;

mod builder;

pub use arc::Arc;
pub use builder::Builder;

/// An immutable set of points that may or may not be connected.
///
/// A single [`Path`] can represent different kinds of 2D shapes!
///
/// [`Path`]: struct.Path.html
#[derive(Debug, Clone)]
pub struct Path {
    raw: lyon::path::Path,
}

impl Path {
    /// Creates a new [`Path`] with the provided closure.
    ///
    /// Use the [`Builder`] to configure your [`Path`].
    ///
    /// [`Path`]: struct.Path.html
    /// [`Builder`]: struct.Builder.html
    pub fn new(f: impl FnOnce(&mut Builder)) -> Self {
        let mut builder = Builder::new();

        // TODO: Make it pure instead of side-effect-based (?)
        f(&mut builder);

        builder.build()
    }

    #[inline]
    pub(crate) fn raw(&self) -> &lyon::path::Path {
        &self.raw
    }

    #[inline]
    pub(crate) fn transformed(
        &self,
        transform: &lyon::math::Transform,
    ) -> Path {
        Path {
            raw: self.raw.transformed(transform),
        }
    }
}
