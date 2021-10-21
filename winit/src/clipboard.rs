//! Access the clipboard.
pub use iced_native::clipboard::Action;

use crate::command::{self, Command};

/// A buffer for short-term storage and transfer within and between
/// applications.
#[allow(missing_debug_implementations)]
#[cfg(target_arch = "wasm32")]
pub struct Clipboard;

#[cfg(target_arch = "wasm32")]
impl Clipboard {
    /// Creates a new [`Clipboard`] for the given window.
    pub fn connect(_window: &winit::window::Window) -> Clipboard {
        Clipboard
    }

    /// Reads the current content of the [`Clipboard`] as text.
    pub fn read(&self) -> Option<String> {
        None
    }

    /// Writes the given text contents to the [`Clipboard`].
    pub fn write(&mut self, _contents: String) {}
}

/// A buffer for short-term storage and transfer within and between
/// applications.
#[allow(missing_debug_implementations)]
#[cfg(not(target_arch = "wasm32"))]
pub struct Clipboard {
    state: State,
}

#[cfg(not(target_arch = "wasm32"))]
enum State {
    Connected(window_clipboard::Clipboard),
    Unavailable,
}

#[cfg(not(target_arch = "wasm32"))]
impl Clipboard {
    /// Creates a new [`Clipboard`] for the given window.
    pub fn connect(window: &winit::window::Window) -> Clipboard {
        let state = window_clipboard::Clipboard::connect(window)
            .ok()
            .map(State::Connected)
            .unwrap_or(State::Unavailable);

        Clipboard { state }
    }

    /// Reads the current content of the [`Clipboard`] as text.
    pub fn read(&self) -> Option<String> {
        match &self.state {
            State::Connected(clipboard) => clipboard.read().ok(),
            State::Unavailable => None,
        }
    }

    /// Writes the given text contents to the [`Clipboard`].
    pub fn write(&mut self, contents: String) {
        match &mut self.state {
            State::Connected(clipboard) => match clipboard.write(contents) {
                Ok(()) => {}
                Err(error) => {
                    log::warn!("error writing to clipboard: {}", error)
                }
            },
            State::Unavailable => {}
        }
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
