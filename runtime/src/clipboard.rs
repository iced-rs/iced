//! Access the clipboard.
use crate::command::{self, Command};
use crate::core::clipboard::Kind;
use crate::futures::MaybeSend;

use std::fmt;

/// A clipboard action to be performed by some [`Command`].
///
/// [`Command`]: crate::Command
pub enum Action<T> {
    /// Read the clipboard and produce `T` with the result.
    Read(Box<dyn Fn(Option<String>) -> T>, Kind),

    /// Write the given contents to the clipboard.
    Write(String, Kind),
}

impl<T> Action<T> {
    /// Maps the output of a clipboard [`Action`] using the provided closure.
    pub fn map<A>(
        self,
        f: impl Fn(T) -> A + 'static + MaybeSend + Sync,
    ) -> Action<A>
    where
        T: 'static,
    {
        match self {
            Self::Read(o, target) => {
                Action::Read(Box::new(move |s| f(o(s))), target)
            }
            Self::Write(content, target) => Action::Write(content, target),
        }
    }
}

impl<T> fmt::Debug for Action<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Read(_, target) => write!(f, "Action::Read{target:?}"),
            Self::Write(_, target) => write!(f, "Action::Write({target:?})"),
        }
    }
}

/// Read the current contents of the clipboard.
pub fn read<Message>(
    f: impl Fn(Option<String>) -> Message + 'static,
) -> Command<Message> {
    Command::single(command::Action::Clipboard(Action::Read(
        Box::new(f),
        Kind::Standard,
    )))
}

/// Read the current contents of the primary clipboard.
pub fn read_primary<Message>(
    f: impl Fn(Option<String>) -> Message + 'static,
) -> Command<Message> {
    Command::single(command::Action::Clipboard(Action::Read(
        Box::new(f),
        Kind::Primary,
    )))
}

/// Write the given contents to the clipboard.
pub fn write<Message>(contents: String) -> Command<Message> {
    Command::single(command::Action::Clipboard(Action::Write(
        contents,
        Kind::Standard,
    )))
}

/// Write the given contents to the primary clipboard.
pub fn write_primary<Message>(contents: String) -> Command<Message> {
    Command::single(command::Action::Clipboard(Action::Write(
        contents,
        Kind::Primary,
    )))
}
