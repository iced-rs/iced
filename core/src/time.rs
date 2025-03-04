//! Keep track of time, both in native and web platforms!

pub use web_time::Duration;
pub use web_time::Instant;
pub use web_time::SystemTime;

/// Creates a [`Duration`] representing the given amount of milliseconds.
pub fn milliseconds(milliseconds: u64) -> Duration {
    Duration::from_millis(milliseconds)
}

/// Creates a [`Duration`] representing the given amount of seconds.
pub fn seconds(seconds: u64) -> Duration {
    Duration::from_secs(seconds)
}

/// Creates a [`Duration`] representing the given amount of minutes.
pub fn minutes(minutes: u64) -> Duration {
    seconds(minutes * 60)
}

/// Creates a [`Duration`] representing the given amount of hours.
pub fn hours(hours: u64) -> Duration {
    minutes(hours * 60)
}

/// Creates a [`Duration`] representing the given amount of days.
pub fn days(days: u64) -> Duration {
    hours(days * 24)
}
