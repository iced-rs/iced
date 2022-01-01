//! Access the clipboard.

#[cfg(feature = "window_clipboard")]
mod window_clipboard;
#[cfg(feature = "window_clipboard")]
pub use self::window_clipboard::*;

#[cfg(not(feature = "window_clipboard"))]
mod null_clipboard;
#[cfg(not(feature = "window_clipboard"))]
pub use null_clipboard::*;
