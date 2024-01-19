//! Handle events of a custom shader widget.
use crate::core::keyboard;
use crate::core::mouse;
use crate::core::time::Instant;
use crate::core::touch;

pub use crate::core::event::Status;

/// A [`Shader`] event.
///
/// [`Shader`]: crate::Shader
#[derive(Debug, Clone, PartialEq)]
pub enum Event {
    /// A mouse event.
    Mouse(mouse::Event),

    /// A touch event.
    Touch(touch::Event),

    /// A keyboard event.
    Keyboard(keyboard::Event),

    /// A window requested a redraw.
    RedrawRequested(Instant),
}
