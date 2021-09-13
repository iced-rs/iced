//! Access the clipboard.
use std::fmt;

/// A buffer for short-term storage and transfer within and between
/// applications.
pub trait Clipboard {
    /// Reads the current content of the [`Clipboard`] as text.
    fn read(&self) -> Option<String>;

    /// Writes the given text contents to the [`Clipboard`].
    fn write(&mut self, contents: String);
}

/// A null implementation of the [`Clipboard`] trait.
#[derive(Debug, Clone, Copy)]
pub struct Null;

impl Clipboard for Null {
    fn read(&self) -> Option<String> {
        None
    }

    fn write(&mut self, _contents: String) {}
}

/// A clipboard action to be performed by some [`Command`].
///
/// [`Command`]: crate::Command
pub enum Action<T> {
    /// Read the clipboard and produce `T` with the result.
    Read(Box<dyn Fn(Option<String>) -> T>),

    /// Write the given contents to the clipboard.
    Write(String),
}

impl<T> Action<T> {
    /// Maps the output of a clipboard [`Action`] using the provided closure.
    pub fn map<A>(self, f: impl Fn(T) -> A + 'static + Send + Sync) -> Action<A>
    where
        T: 'static,
    {
        match self {
            Self::Read(o) => Action::Read(Box::new(move |s| f(o(s)))),
            Self::Write(content) => Action::Write(content),
        }
    }
}

impl<T> fmt::Debug for Action<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Read(_) => write!(f, "Action::Read"),
            Self::Write(_) => write!(f, "Action::Write"),
        }
    }
}
