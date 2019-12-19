/// A buffer for short-term storage and transfer within and between
/// applications.
pub trait Clipboard {
    /// Returns the current content of the [`Clipboard`] as text.
    ///
    /// [`Clipboard`]: trait.Clipboard.html
    fn content(&self) -> Option<String>;
}
