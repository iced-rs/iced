//! Listen to input method events.

/// A input method event.
///
/// _**Note:** This type is largely incomplete! If you need to track
/// additional events, feel free to [open an issue] and share your use case!_
///
/// [open an issue]: https://github.com/iced-rs/iced/issues
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Event {
    // These events correspond to underlying winit ime events.
    // https://docs.rs/winit/latest/winit/event/enum.Ime.html
    /// the IME was enabled.
    Enabled,

    /// new composing text should be set at the cursor position.
    Preedit(String, Option<(usize, usize)>),

    /// text should be inserted into the editor widget.
    Commit(String),

    /// the IME was disabled.
    Disabled,
}
