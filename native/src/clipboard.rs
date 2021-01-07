/// A buffer for short-term storage and transfer within and between
/// applications.
pub trait Clipboard {
    /// Returns the current content of the [`Clipboard`] as text.
    fn content(&self) -> Option<String>;

    /// Set IME window's position.
    fn set_ime_position(&self, position: iced_core::Point);
}
