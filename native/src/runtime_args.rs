//! Optional runtime arguments to runtime application
use std::time::Duration;

#[derive(Debug)]
/// A list of events that are provided to the application.
///
/// Currently the only trace supported is composed of "events" of type ([`Message`], [`Duration`]),
///
/// Each event's [`Message`] is provided to the application after [`Duration`] time has passed
/// since the application began
///
/// Currently, when all events in a trace have been exhausted the application will be signaled to exit after a 50ms delay (this delay is arbitrary).
///
/// This type can be extended to include other trace variants such as [`Event`]s
///
/// [`Duration`]: std::time::Duration
/// [`Event`]: crate::event::Event
pub enum Trace<Message> {
    /// Trace composed of [`Application`] level Messages
    MessageTrace(Vec<(Message, Duration)>),

    /// If no trace is provided, we have the null variant. This is to future proof the
    /// [`RuntimeArgs`] object against cases where no trace is provided with [`RuntimeArgs`\
    Null,
}

///Optional arguments that are passed to a native [`Application`] at run time
#[derive(Debug)]
pub struct RuntimeArgs<Message> {
    /// Trace object
    pub trace: Trace<Message>,
}

impl<Message> RuntimeArgs<Message> {
    /// Creates a new [`RuntimeArgs`] object
    pub fn new() -> Self {
        Self { trace: Trace::Null }
    }

    /// Sets the [`Trace`] to [`Trace::MessageTrace`]
    pub fn message_trace(
        mut self,
        message_trace: Vec<(Message, Duration)>,
    ) -> Self {
        self.trace = Trace::MessageTrace(message_trace);
        self
    }
}
