//! Access the clipboard.
pub use iced_runtime::clipboard::Action;

use iced_runtime::command::{self, Command};
use std::ffi::c_void;
use std::sync::{Arc, Mutex};

/// A buffer for short-term storage and transfer within and between
/// applications.
#[allow(missing_debug_implementations)]
pub struct Clipboard {
    pub(crate) state: State,
}

pub(crate) enum State {
    Connected(Arc<Mutex<smithay_clipboard::Clipboard>>),
    Unavailable,
}

impl Clipboard {
    pub unsafe fn connect(display: *mut c_void) -> Clipboard {
        let context = Arc::new(Mutex::new(smithay_clipboard::Clipboard::new(
            display as *mut _,
        )));

        Clipboard {
            state: State::Connected(context),
        }
    }

    /// Creates a new [`Clipboard`] that isn't associated with a window.
    /// This clipboard will never contain a copied value.
    pub fn unconnected() -> Clipboard {
        Clipboard {
            state: State::Unavailable,
        }
    }

    /// Reads the current content of the [`Clipboard`] as text.
    pub fn read(&self) -> Option<String> {
        match &self.state {
            State::Connected(clipboard) => {
                let clipboard = clipboard.lock().unwrap();
                clipboard.load().ok()
            }
            State::Unavailable => None,
        }
    }

    /// Writes the given text contents to the [`Clipboard`].
    pub fn write(&mut self, contents: String) {
        match &mut self.state {
            State::Connected(clipboard) => {
                clipboard.lock().unwrap().store(contents)
            }
            State::Unavailable => {}
        }
    }
}

impl iced_runtime::core::clipboard::Clipboard for Clipboard {
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
