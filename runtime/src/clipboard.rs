//! Access the clipboard.
use crate::core::clipboard::Kind;
use crate::futures::futures::channel::oneshot;
use crate::task::{self, Task};

/// A clipboard action to be performed by some [`Task`].
///
/// [`Task`]: crate::Task
#[derive(Debug)]
pub enum Action {
    /// Read the clipboard and produce `T` with the result.
    Read {
        /// The clipboard target.
        target: Kind,
        /// The channel to send the read contents.
        channel: oneshot::Sender<Option<String>>,
    },

    /// Write the given contents to the clipboard.
    Write {
        /// The clipboard target.
        target: Kind,
        /// The contents to be written.
        contents: String,
    },
}

/// Read the current contents of the clipboard.
pub fn read() -> Task<Option<String>> {
    task::oneshot(|channel| {
        crate::Action::Clipboard(Action::Read {
            target: Kind::Standard,
            channel,
        })
    })
}

/// Read the current contents of the primary clipboard.
pub fn read_primary() -> Task<Option<String>> {
    task::oneshot(|channel| {
        crate::Action::Clipboard(Action::Read {
            target: Kind::Primary,
            channel,
        })
    })
}

/// Write the given contents to the clipboard.
pub fn write<T>(contents: String) -> Task<T> {
    task::effect(crate::Action::Clipboard(Action::Write {
        target: Kind::Standard,
        contents,
    }))
}

/// Write the given contents to the primary clipboard.
pub fn write_primary<Message>(contents: String) -> Task<Message> {
    task::effect(crate::Action::Clipboard(Action::Write {
        target: Kind::Primary,
        contents,
    }))
}
