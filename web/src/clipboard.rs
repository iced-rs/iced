/// A buffer for short-term storage and transfer within and between
/// applications.
#[derive(Debug, Clone, Copy)]
pub struct Clipboard;

impl Clipboard {
    /// Creates a new [`Clipboard`].
    pub fn new() -> Self {
        Self
    }

    /// Reads the current content of the [`Clipboard`] as text.
    pub fn read(&self) -> Option<String> {
        unimplemented! {}
    }

    /// Writes the given text contents to the [`Clipboard`].
    pub fn write(&mut self, _contents: String) {
        unimplemented! {}
    }
}
