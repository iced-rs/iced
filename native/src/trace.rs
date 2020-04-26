//! fixme: looks like that's something which would be implemented by a nice crate already
use std::{collections::VecDeque, time};

/*#[derive(Debug)]
struct TimeBuffer {
    head: usize,
    size: usize,
    contents: Vec<time::Duration>, // What's wrong with Deque ?
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

impl Default for TimeBuffer { fn default() -> Self { Self::new(200) } }*/

type Profile = VecDeque<time::Duration>;

/// Time scope execution
#[derive(Debug)]
pub struct ProfileScope<'t>{
    profile: &'t mut Profile,
    start: time::Instant,
}

impl<'t> Drop for ProfileScope<'t> {
    fn drop(&mut self) {
        self.profile.push_back(time::Instant::now() - self.start);
    }
}

use enum_iterator::IntoEnumIterator;
/// Split profile into components
#[derive(IntoEnumIterator,Debug)]
pub enum Component {
/// Until first request
    Setup,
    //Event, Update, View, Layout, Draw, Render,
    ///
    _Length
}

/// Collects application execution time profile and log messages
#[derive(Default,Debug)]
pub struct Trace {
    is_enabled: bool,

    profile: Vec<Profile>,

    message_count: usize,
    last_messages: VecDeque<String>,
}

impl Trace {
    /// Creates an empty profile
    pub fn new() -> Self { Self{ profile: { let mut vec = Vec::new(); vec.resize_with(Component::_Length as usize, Default::default); vec }, ..Self::default() } }

    /// Toggle profiling
    pub fn toggle(&mut self) {
        self.is_enabled = !self.is_enabled;
    }

    /// Appends scope execution time to profile
    pub fn scope(&mut self, component: Component) -> ProfileScope<'_> { ProfileScope{profile: &mut self.profile[component as usize], start: time::Instant::now()} }

    /// Appends message to log
    pub fn log_message<Message: std::fmt::Debug>(&mut self, message: &Message) {
        self.last_messages.push_back(format!("{:?}", message));

        if self.last_messages.len() > 10 {
            let _ = self.last_messages.pop_front();
        }

        self.message_count += 1;
    }

    /// Formats into lines for display
    pub fn lines(&self) -> Vec<String> {
        if !self.is_enabled {
            return Vec::new();
        }

        let mut lines = Vec::new();

        lines.push(format!(
            "{} {} - {}",
            env!("CARGO_PKG_NAME"),
            env!("CARGO_PKG_VERSION"),
            env!("CARGO_PKG_REPOSITORY"),
        ));
        lines.extend(
            Component::into_enum_iter().zip(self.profile.iter()).map(|(component, profile)| {
                let average = |v:&Profile| -> std::time::Duration { v.iter().sum::<time::Duration>() / (v.len() as u32) };
                format!("{:?}: {:?}", component, average(profile))
            })
        );
        lines.extend(
            self.last_messages.iter().map(|msg| format!("    {}", msg)),
        );
        lines
    }
}
