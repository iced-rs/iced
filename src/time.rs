//! Listen and react to time.
pub use crate::runtime::time::{Duration, Instant};

use crate::Subscription;

/// Returns a [`Subscription`] that produces messages at a set interval.
///
/// The first message is produced after a `duration`, and then continues to
/// produce more messages every `duration` after that.
pub fn every(duration: Duration) -> Subscription<Instant> {
    iced_futures::time::every(duration)
}
