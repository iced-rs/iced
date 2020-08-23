/// A buffer for short-term storage and transfer within and between
/// applications.
pub trait Clipboard {
    /// Returns the current content of the [`Clipboard`] as text.
    ///
    /// [`Clipboard`]: trait.Clipboard.html
    fn content(&self) -> Option<String>;
}

#[cfg(feature = "clipboard")]
use ::clipboard::{ClipboardContext, ClipboardProvider};
#[cfg(feature = "clipboard")]
pub fn copy_to_clipboard<S: Into<String>>(text: S) {
    let mut ctx: ClipboardContext = ClipboardProvider::new().unwrap();
    ctx.set_contents(text.into()).unwrap();
}
