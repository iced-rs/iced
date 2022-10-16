//! Handle events of a canvas.
use iced_native::keyboard;
use iced_native::mouse;
use iced_native::touch;

pub use iced_native::event::Status;

/// A [`Canvas`] event.
///
/// [`Canvas`]: crate::widget::Canvas
#[derive(Debug, Clone, PartialEq)]
pub enum Event {
    /// A mouse event.
    Mouse(mouse::Event),

    /// A touch event.
    Touch(touch::Event),

    /// A keyboard event.
    Keyboard(keyboard::Event),
}
