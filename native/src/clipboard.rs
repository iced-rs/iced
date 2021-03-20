//! Access the clipboard.

/// A buffer for short-term storage and transfer within and between
/// applications.
pub trait Clipboard {
    /// Reads the current content of the [`Clipboard`] as text.
    fn read(&self) -> Option<String>;

    /// Writes the given text contents to the [`Clipboard`].
    fn write(&mut self, contents: String);

    /// Set IME window's position.
    fn set_ime_position(&self, position: iced_core::Point);
}

/// A null implementation of the [`Clipboard`] trait.
#[derive(Debug, Clone, Copy)]
pub struct Null;

impl Clipboard for Null {
    fn read(&self) -> Option<String> {
        None
    }

    fn write(&mut self, _contents: String) {}

    fn set_ime_position(&self, _position: iced_core::Point) {}
}
