//! Traits for the message type of a [`Program`](crate::Program).

/// A trait alias for [`Clone`], but only when the `time-travel`
/// feature is enabled.
#[cfg(feature = "time-travel")]
pub trait MaybeClone: Clone {}

#[cfg(feature = "time-travel")]
impl<T> MaybeClone for T where T: Clone {}

/// A trait alias for [`Clone`], but only when the `time-travel`
/// feature is enabled.
#[cfg(not(feature = "time-travel"))]
pub trait MaybeClone {}

#[cfg(not(feature = "time-travel"))]
impl<T> MaybeClone for T {}

/// A trait alias for [`Debug`](std::fmt::Debug), but only when the
/// `debug` feature is enabled.
#[cfg(feature = "debug")]
pub trait MaybeDebug: std::fmt::Debug {}

#[cfg(feature = "debug")]
impl<T> MaybeDebug for T where T: std::fmt::Debug {}

/// A trait alias for [`Debug`](std::fmt::Debug), but only when the
/// `debug` feature is enabled.
#[cfg(not(feature = "debug"))]
pub trait MaybeDebug {}

#[cfg(not(feature = "debug"))]
impl<T> MaybeDebug for T {}
