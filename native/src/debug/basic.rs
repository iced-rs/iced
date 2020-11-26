#![allow(missing_docs)]
use std::{collections::VecDeque, time};

/// A bunch of time measurements for debugging purposes.
#[derive(Debug)]
pub struct Debug {
    is_enabled: bool,

    startup_start: time::Instant,
    startup_duration: time::Duration,

    update_start: time::Instant,
    update_durations: TimeBuffer,

    view_start: time::Instant,
    view_durations: TimeBuffer,

    layout_start: time::Instant,
    layout_durations: TimeBuffer,

    event_start: time::Instant,
    event_durations: TimeBuffer,

    draw_start: time::Instant,
    draw_durations: TimeBuffer,

    render_start: time::Instant,
    render_durations: TimeBuffer,

    message_count: usize,
    last_messages: VecDeque<String>,
}

impl Debug {
    /// Creates a new [`struct@Debug`].
    pub fn new() -> Self {
        let now = time::Instant::now();

        Self {
            is_enabled: false,
            startup_start: now,
            startup_duration: time::Duration::from_secs(0),

            update_start: now,
            update_durations: TimeBuffer::new(200),

            view_start: now,
            view_durations: TimeBuffer::new(200),

            layout_start: now,
            layout_durations: TimeBuffer::new(200),

            event_start: now,
            event_durations: TimeBuffer::new(200),

            draw_start: now,
            draw_durations: TimeBuffer::new(200),

            render_start: now,
            render_durations: TimeBuffer::new(50),

            message_count: 0,
            last_messages: VecDeque::new(),
        }
    }

    pub fn toggle(&mut self) {
        self.is_enabled = !self.is_enabled;
    }

    pub fn startup_started(&mut self) {
        self.startup_start = time::Instant::now();
    }

    pub fn startup_finished(&mut self) {
        self.startup_duration = time::Instant::now() - self.startup_start;
    }

    pub fn update_started(&mut self) {
        self.update_start = time::Instant::now();
    }

    pub fn update_finished(&mut self) {
        self.update_durations
            .push(time::Instant::now() - self.update_start);
    }

    pub fn view_started(&mut self) {
        self.view_start = time::Instant::now();
    }

    pub fn view_finished(&mut self) {
        self.view_durations
            .push(time::Instant::now() - self.view_start);
    }

    pub fn layout_started(&mut self) {
        self.layout_start = time::Instant::now();
    }

    pub fn layout_finished(&mut self) {
        self.layout_durations
            .push(time::Instant::now() - self.layout_start);
    }

    pub fn event_processing_started(&mut self) {
        self.event_start = time::Instant::now();
    }

    pub fn event_processing_finished(&mut self) {
        self.event_durations
            .push(time::Instant::now() - self.event_start);
    }

    pub fn draw_started(&mut self) {
        self.draw_start = time::Instant::now();
    }

    pub fn draw_finished(&mut self) {
        self.draw_durations
            .push(time::Instant::now() - self.draw_start);
    }

    pub fn render_started(&mut self) {
        self.render_start = time::Instant::now();
    }

    pub fn render_finished(&mut self) {
        self.render_durations
            .push(time::Instant::now() - self.render_start);
    }

    pub fn log_message<Message: std::fmt::Debug>(&mut self, message: &Message) {
        self.last_messages.push_back(format!("{:?}", message));

        if self.last_messages.len() > 10 {
            let _ = self.last_messages.pop_front();
        }

        self.message_count += 1;
    }

    pub fn overlay(&self) -> Vec<String> {
        if !self.is_enabled {
            return Vec::new();
        }

        let mut lines = Vec::new();

        fn key_value<T: std::fmt::Debug>(key: &str, value: T) -> String {
            format!("{} {:?}", key, value)
        }

        lines.push(format!(
            "{} {} - {}",
            env!("CARGO_PKG_NAME"),
            env!("CARGO_PKG_VERSION"),
            env!("CARGO_PKG_REPOSITORY"),
        ));
        lines.push(key_value("Startup:", self.startup_duration));
        lines.push(key_value("Update:", self.update_durations.average()));
        lines.push(key_value("View:", self.view_durations.average()));
        lines.push(key_value("Layout:", self.layout_durations.average()));
        lines.push(key_value(
            "Event processing:",
            self.event_durations.average(),
        ));
        lines.push(key_value(
            "Primitive generation:",
            self.draw_durations.average(),
        ));
        lines.push(key_value("Render:", self.render_durations.average()));
        lines.push(key_value("Message count:", self.message_count));
        lines.push(String::from("Last messages:"));
        lines.extend(self.last_messages.iter().map(|msg| {
            if msg.len() <= 100 {
                format!("    {}", msg)
            } else {
                format!("    {:.100}...", msg)
            }
        }));

        lines
    }
}

#[derive(Debug)]
struct TimeBuffer {
    head: usize,
    size: usize,
    contents: Vec<time::Duration>,
}

impl TimeBuffer {
    fn new(capacity: usize) -> TimeBuffer {
        TimeBuffer {
            head: 0,
            size: 0,
            contents: vec![time::Duration::from_secs(0); capacity],
        }
    }

    fn push(&mut self, duration: time::Duration) {
        self.head = (self.head + 1) % self.contents.len();
        self.contents[self.head] = duration;
        self.size = (self.size + 1).min(self.contents.len());
    }

    fn average(&self) -> time::Duration {
        let sum: time::Duration = if self.size == self.contents.len() {
            self.contents[..].iter().sum()
        } else {
            self.contents[..self.size].iter().sum()
        };

        sum / self.size.max(1) as u32
    }
}
