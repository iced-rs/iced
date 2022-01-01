//! Access the clipboard.
pub use iced_native::clipboard::Action;

use crate::command::{self, Command};

/// A buffer for short-term storage and transfer within and between
/// applications.
#[derive(Debug)]
pub struct Clipboard;

impl Clipboard {
    /// Creates a new [`Clipboard`] for the given window.
    pub fn new() -> Clipboard {
        Clipboard
    }

    /// Reads the current content of the [`Clipboard`] as text.
    pub fn read(&self) -> Option<String> {
        None
    }

    /// Writes the given text contents to the [`Clipboard`].
    pub fn write(&mut self, _contents: String) {
        log::warn!("cannot write to null clipboard");
    }
}

impl iced_native::Clipboard for Clipboard {
    fn read(&self) -> Option<String> {
        self.read()
    }

    fn write(&mut self, contents: String) {
        self.write(contents)
    }
}

/// Read the current contents of the clipboard.
pub fn read<Message>(
    f: impl Fn(Option<String>) -> Message + 'static,
) -> Command<Message> {
    Command::single(command::Action::Clipboard(Action::Read(Box::new(f))))
}

/// Write the given contents to the clipboard.
pub fn write<Message>(contents: String) -> Command<Message> {
    Command::single(command::Action::Clipboard(Action::Write(contents)))
}
