//! Handle events of a canvas.
use crate::core::keyboard;
use crate::core::mouse;
use crate::core::touch;

pub use crate::core::event::Status;

/// A [`Canvas`] event.
///
/// [`Canvas`]: crate::widget::Canvas
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Event {
    /// A mouse event.
    Mouse(mouse::Event),

    /// A touch event.
    Touch(touch::Event),

    /// A keyboard event.
    Keyboard(keyboard::Event),
}
