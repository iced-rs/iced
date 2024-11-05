//! Handle events of a user interface.
use crate::keyboard;
use crate::mouse;
use crate::touch;
use crate::window;

/// A user interface event.
///
/// _**Note:** This type is largely incomplete! If you need to track
/// additional events, feel free to [open an issue] and share your use case!_
///
/// [open an issue]: https://github.com/iced-rs/iced/issues
#[derive(Debug, Clone, PartialEq)]
pub enum Event {
    /// A keyboard event
    Keyboard(keyboard::Event),

    /// A mouse event
    Mouse(mouse::Event),

    /// A window event
    Window(window::Event),

    /// A touch event
    Touch(touch::Event),

    /// Special platform event
    Special(Box<dyn SpecialEvent + Send>),
}

/// Define the trait without Clone and PartialEq
pub trait SpecialEvent: std::fmt::Debug + std::any::Any {
    /// check if it is equal
    fn equal(&self, other: &dyn SpecialEvent) -> bool;

    /// how to downcast self as any
    fn as_any(&self) -> &dyn std::any::Any;

    /// We add a `clone_box` method for cloning trait objects
    fn clone_box(&self) -> Box<dyn SpecialEvent + Send>;
}

// Implement `CustomInner` for `Clone`
impl Clone for Box<dyn SpecialEvent + Send> {
    fn clone(&self) -> Box<dyn SpecialEvent + Send> {
        self.clone_box()
    }
}

// Example implementation for PartialEq
impl PartialEq for Box<dyn SpecialEvent + Send> {
    fn eq(&self, other: &Self) -> bool {
        self.equal(&(**other)) // Customize this equality check as needed
    }
}

impl dyn SpecialEvent {
    /// down cast self to other type
    pub fn downcast_ref<T: 'static>(&self) -> Option<&T> {
        self.as_any().downcast_ref::<T>()
    }
}

/// The status of an [`Event`] after being processed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    /// The [`Event`] was **NOT** handled by any widget.
    Ignored,

    /// The [`Event`] was handled and processed by a widget.
    Captured,
}

impl Status {
    /// Merges two [`Status`] into one.
    ///
    /// `Captured` takes precedence over `Ignored`:
    ///
    /// ```
    /// use iced_core::event::Status;
    ///
    /// assert_eq!(Status::Ignored.merge(Status::Ignored), Status::Ignored);
    /// assert_eq!(Status::Ignored.merge(Status::Captured), Status::Captured);
    /// assert_eq!(Status::Captured.merge(Status::Ignored), Status::Captured);
    /// assert_eq!(Status::Captured.merge(Status::Captured), Status::Captured);
    /// ```
    pub fn merge(self, b: Self) -> Self {
        match self {
            Status::Ignored => b,
            Status::Captured => Status::Captured,
        }
    }
}
