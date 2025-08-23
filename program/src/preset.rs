use crate::runtime::Task;

use std::borrow::Cow;
use std::fmt;

/// A specific boot strategy for a [`Program`](crate::Program).
pub struct Preset<State, Message> {
    name: Cow<'static, str>,
    boot: Box<dyn Fn() -> (State, Task<Message>)>,
}

impl<State, Message> Preset<State, Message> {
    /// Creates a new [`Preset`] with the given name and boot strategy.
    pub fn new(
        name: impl Into<Cow<'static, str>>,
        boot: impl Fn() -> (State, Task<Message>) + 'static,
    ) -> Self {
        Self {
            name: name.into(),
            boot: Box::new(boot),
        }
    }

    /// Returns the name of the [`Preset`].
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Boots the [`Preset`], returning the initial [`Program`](crate::Program) state and
    /// a [`Task`] for concurrent booting.
    pub fn boot(&self) -> (State, Task<Message>) {
        (self.boot)()
    }
}

impl<State, Message> fmt::Debug for Preset<State, Message> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Preset")
            .field("name", &self.name)
            .finish_non_exhaustive()
    }
}
