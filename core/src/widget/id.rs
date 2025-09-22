use std::borrow;
use std::sync::atomic::{self, AtomicUsize};

static NEXT_ID: AtomicUsize = AtomicUsize::new(0);

/// The identifier of a generic widget.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Id(Internal);

impl Id {
    /// Creates a custom [`Id`].
    pub fn new(id: impl Into<borrow::Cow<'static, str>>) -> Self {
        Self(Internal::Custom(id.into()))
    }

    /// Creates a unique [`Id`].
    ///
    /// This function produces a different [`Id`] every time it is called.
    pub fn unique() -> Self {
        let id = NEXT_ID.fetch_add(1, atomic::Ordering::Relaxed);

        Self(Internal::Unique(id))
    }
}

impl From<&'static str> for Id {
    fn from(value: &'static str) -> Self {
        Self::new(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum Internal {
    Unique(usize),
    Custom(borrow::Cow<'static, str>),
}

#[cfg(test)]
mod tests {
    use super::Id;

    #[test]
    fn unique_generates_different_ids() {
        let a = Id::unique();
        let b = Id::unique();

        assert_ne!(a, b);
    }
}
