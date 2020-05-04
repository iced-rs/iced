use iced_native::mouse;

/// A [`Canvas`] event.
///
/// [`Canvas`]: struct.Event.html
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Event {
    /// A mouse event.
    Mouse(mouse::Event),
}
