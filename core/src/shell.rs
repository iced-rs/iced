use crate::InputMethod;
use crate::event;
use crate::window;

/// A connection to the state of a shell.
///
/// A [`Widget`] can leverage a [`Shell`] to trigger changes in an application,
/// like publishing messages or invalidating the current layout.
///
/// [`Widget`]: crate::Widget
#[derive(Debug)]
pub struct Shell<'a, Message> {
    messages: &'a mut Vec<Message>,
    event_status: event::Status,
    redraw_request: window::RedrawRequest,
    input_method: InputMethod,
    is_layout_invalid: bool,
    are_widgets_invalid: bool,
}

impl<'a, Message> Shell<'a, Message> {
    /// Creates a new [`Shell`] with the provided buffer of messages.
    pub fn new(messages: &'a mut Vec<Message>) -> Self {
        Self {
            messages,
            event_status: event::Status::Ignored,
            redraw_request: window::RedrawRequest::Wait,
            is_layout_invalid: false,
            are_widgets_invalid: false,
            input_method: InputMethod::Disabled,
        }
    }

    /// Returns true if the [`Shell`] contains no published messages
    pub fn is_empty(&self) -> bool {
        self.messages.is_empty()
    }

    /// Publish the given `Message` for an application to process it.
    pub fn publish(&mut self, message: Message) {
        self.messages.push(message);
    }

    /// Marks the current event as captured. Prevents "event bubbling".
    ///
    /// A widget should capture an event when no ancestor should
    /// handle it.
    pub fn capture_event(&mut self) {
        self.event_status = event::Status::Captured;
    }

    /// Returns the current [`event::Status`] of the [`Shell`].
    pub fn event_status(&self) -> event::Status {
        self.event_status
    }

    /// Returns whether the current event has been captured.
    pub fn is_event_captured(&self) -> bool {
        self.event_status == event::Status::Captured
    }

    /// Requests a new frame to be drawn as soon as possible.
    pub fn request_redraw(&mut self) {
        self.redraw_request = window::RedrawRequest::NextFrame;
    }

    /// Requests a new frame to be drawn at the given [`window::RedrawRequest`].
    pub fn request_redraw_at(
        &mut self,
        redraw_request: impl Into<window::RedrawRequest>,
    ) {
        self.redraw_request = self.redraw_request.min(redraw_request.into());
    }

    /// Returns the request a redraw should happen, if any.
    pub fn redraw_request(&self) -> window::RedrawRequest {
        self.redraw_request
    }

    /// Replaces the redraw request of the [`Shell`]; without conflict resolution.
    ///
    /// This is useful if you want to overwrite the redraw request to a previous value.
    /// Since it's a fairly advanced use case and should rarely be used, it is a static
    /// method.
    pub fn replace_redraw_request(
        shell: &mut Self,
        redraw_request: window::RedrawRequest,
    ) {
        shell.redraw_request = redraw_request;
    }

    /// Requests the current [`InputMethod`] strategy.
    ///
    /// __Important__: This request will only be honored by the
    /// [`Shell`] only during a [`window::Event::RedrawRequested`].
    pub fn request_input_method<T: AsRef<str>>(
        &mut self,
        ime: &InputMethod<T>,
    ) {
        self.input_method.merge(ime);
    }

    /// Returns the current [`InputMethod`] strategy.
    pub fn input_method(&self) -> &InputMethod {
        &self.input_method
    }

    /// Returns the current [`InputMethod`] strategy.
    pub fn input_method_mut(&mut self) -> &mut InputMethod {
        &mut self.input_method
    }

    /// Returns whether the current layout is invalid or not.
    pub fn is_layout_invalid(&self) -> bool {
        self.is_layout_invalid
    }

    /// Invalidates the current application layout.
    ///
    /// The shell will relayout the application widgets.
    pub fn invalidate_layout(&mut self) {
        self.is_layout_invalid = true;
    }

    /// Triggers the given function if the layout is invalid, cleaning it in the
    /// process.
    pub fn revalidate_layout(&mut self, f: impl FnOnce()) {
        if self.is_layout_invalid {
            self.is_layout_invalid = false;

            f();
        }
    }

    /// Returns whether the widgets of the current application have been
    /// invalidated.
    pub fn are_widgets_invalid(&self) -> bool {
        self.are_widgets_invalid
    }

    /// Invalidates the current application widgets.
    ///
    /// The shell will rebuild and relayout the widget tree.
    pub fn invalidate_widgets(&mut self) {
        self.are_widgets_invalid = true;
    }

    /// Merges the current [`Shell`] with another one by applying the given
    /// function to the messages of the latter.
    ///
    /// This method is useful for composition.
    pub fn merge<B>(&mut self, other: Shell<'_, B>, f: impl Fn(B) -> Message) {
        self.messages.extend(other.messages.drain(..).map(f));

        self.is_layout_invalid =
            self.is_layout_invalid || other.is_layout_invalid;

        self.are_widgets_invalid =
            self.are_widgets_invalid || other.are_widgets_invalid;

        self.redraw_request = self.redraw_request.min(other.redraw_request);
        self.event_status = self.event_status.merge(other.event_status);
        self.input_method.merge(&other.input_method);
    }
}
