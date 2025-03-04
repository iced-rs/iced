use crate::core::event;
use crate::core::time::Instant;
use crate::core::window;

/// A runtime action that can be performed by some widgets.
#[derive(Debug, Clone)]
pub struct Action<Message> {
    message_to_publish: Option<Message>,
    redraw_request: window::RedrawRequest,
    event_status: event::Status,
}

impl<Message> Action<Message> {
    fn new() -> Self {
        Self {
            message_to_publish: None,
            redraw_request: window::RedrawRequest::Wait,
            event_status: event::Status::Ignored,
        }
    }

    /// Creates a new "capturing" [`Action`]. A capturing [`Action`]
    /// will make other widgets consider it final and prevent further
    /// processing.
    ///
    /// Prevents "event bubbling".
    pub fn capture() -> Self {
        Self {
            event_status: event::Status::Captured,
            ..Self::new()
        }
    }

    /// Creates a new [`Action`] that publishes the given `Message` for
    /// the application to handle.
    ///
    /// Publishing a `Message` always produces a redraw.
    pub fn publish(message: Message) -> Self {
        Self {
            message_to_publish: Some(message),
            ..Self::new()
        }
    }

    /// Creates a new [`Action`] that requests a redraw to happen as
    /// soon as possible; without publishing any `Message`.
    pub fn request_redraw() -> Self {
        Self {
            redraw_request: window::RedrawRequest::NextFrame,
            ..Self::new()
        }
    }

    /// Creates a new [`Action`] that requests a redraw to happen at
    /// the given [`Instant`]; without publishing any `Message`.
    ///
    /// This can be useful to efficiently animate content, like a
    /// blinking caret on a text input.
    pub fn request_redraw_at(at: Instant) -> Self {
        Self {
            redraw_request: window::RedrawRequest::At(at),
            ..Self::new()
        }
    }

    /// Marks the [`Action`] as "capturing". See [`Self::capture`].
    pub fn and_capture(mut self) -> Self {
        self.event_status = event::Status::Captured;
        self
    }

    /// Converts the [`Action`] into its internal parts.
    ///
    /// This method is meant to be used by runtimes, libraries, or internal
    /// widget implementations.
    pub fn into_inner(
        self,
    ) -> (Option<Message>, window::RedrawRequest, event::Status) {
        (
            self.message_to_publish,
            self.redraw_request,
            self.event_status,
        )
    }
}
