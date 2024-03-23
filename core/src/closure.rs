//! Box closures with ease.
//!
//! These are just a bunch of types that wrap boxed closures with
//! blanket [`From`] implementations for easy conversions.
//!
//! Mainly, it allows functions to take `Into<T>` where `T` may end
//! up being a boxed closure.

/// A boxed closure that takes `A` by reference and produces `O`.
#[allow(missing_debug_implementations)]
pub struct Unary<'a, A, O>(pub Box<dyn Fn(&A) -> O + 'a>);

impl<'a, A, O, T> From<T> for Unary<'a, A, O>
where
    T: Fn(&A) -> O + 'a,
{
    fn from(f: T) -> Self {
        Self(Box::new(f))
    }
}

/// A boxed closure that takes `A` by reference and `B` by value and produces `O`.
#[allow(missing_debug_implementations)]
pub struct Binary<'a, A, B, O>(pub Box<dyn Fn(&A, B) -> O + 'a>);

impl<'a, A, B, O, T> From<T> for Binary<'a, A, B, O>
where
    T: Fn(&A, B) -> O + 'a,
{
    fn from(f: T) -> Self {
        Self(Box::new(f))
    }
}
