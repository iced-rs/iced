//! Access the clipboard.

/// A buffer for short-term storage and transfer within and between
/// applications.
pub trait Clipboard {
    /// Reads the current content of the [`Clipboard`] as text.
    fn read(&self, kind: Kind) -> Option<String>;

    /// Writes the given text contents to the [`Clipboard`].
    fn write(&mut self, kind: Kind, contents: String);
}

/// The kind of [`Clipboard`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    /// The standard clipboard.
    Standard,
    /// The primary clipboard.
    ///
    /// Normally only present in X11 and Wayland.
    Primary,
}

/// A null implementation of the [`Clipboard`] trait.
#[derive(Debug, Clone, Copy)]
pub struct Null;

impl Clipboard for Null {
    fn read(&self, _kind: Kind) -> Option<String> {
        None
    }

    fn write(&mut self, _kind: Kind, _contents: String) {}
}
