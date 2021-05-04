//! Listen and react to time.
use crate::Subscription;

/// Returns a [`Subscription`] that produces messages at a set interval.
///
/// The first message is produced after a `duration`, and then continues to
/// produce more messages every `duration` after that.
pub fn every(
    duration: std::time::Duration,
) -> Subscription<std::time::Instant> {
    iced_futures::time::every(duration)
}
