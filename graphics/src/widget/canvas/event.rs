//! Handle events of a canvas.
use iced_native::keyboard;
use iced_native::mouse;

pub use iced_native::event::Status;

/// A [`Canvas`] event.
///
/// [`Canvas`]: crate::widget::Canvas
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Event {
    /// A mouse event.
    Mouse(mouse::Event),

    /// A keyboard event.
    Keyboard(keyboard::Event),
}
