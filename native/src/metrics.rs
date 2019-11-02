use std::time;

/// A bunch of metrics about an Iced application.
#[derive(Debug, Clone, Copy)]
pub struct Metrics {
    pub startup_time: time::Duration,
    pub update_time: time::Duration,
    pub view_time: time::Duration,
    pub renderer_output_time: time::Duration,
    pub message_count: usize,
}
