/// A IME event.
///
/// _**Note:** This type is largely incomplete! If you need to track
/// additional events, feel free to [open an issue] and share your use case!_
///
/// [open an issue]: https://github.com/iced-rs/iced/issues
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Event {
    /// IME enabled.
    IMEEnabled,

    /// selecting input.
    IMEPreedit(String, Option<(usize, usize)>),

    /// enter input.
    IMECommit(String),
}
