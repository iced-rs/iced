/// A connection to the state of a shell.
///
/// A [`Widget`] can leverage a [`Shell`] to trigger changes in an application,
/// like publishing messages or invalidating the current layout.
///
/// [`Widget`]: crate::Widget
#[derive(Debug)]
pub struct Shell<'a, Message> {
    messages: &'a mut Vec<Message>,
    is_layout_invalid: bool,
}

impl<'a, Message> Shell<'a, Message> {
    /// Creates a new [`Shell`] with the provided buffer of messages.
    pub fn new(messages: &'a mut Vec<Message>) -> Self {
        Self {
            messages,
            is_layout_invalid: false,
        }
    }

    /// Triggers the given function if the layout is invalid, cleaning it in the
    /// process.
    pub fn with_invalid_layout(&mut self, f: impl FnOnce()) {
        if self.is_layout_invalid {
            self.is_layout_invalid = false;

            f()
        }
    }

    /// Publish the given `Message` for an application to process it.
    pub fn publish(&mut self, message: Message) {
        self.messages.push(message);
    }

    /// Invalidates the current application layout.
    ///
    /// The shell will relayout the application widgets.
    pub fn invalidate_layout(&mut self) {
        self.is_layout_invalid = true;
    }

    /// Merges the current [`Shell`] with another one by applying the given
    /// function to the messages of the latter.
    ///
    /// This method is useful for composition.
    pub fn merge<B>(&mut self, other: Shell<'_, B>, f: impl Fn(B) -> Message) {
        self.messages.extend(other.messages.drain(..).map(f));

        self.is_layout_invalid =
            self.is_layout_invalid || other.is_layout_invalid;
    }
}
