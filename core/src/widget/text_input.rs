//! Display fields that can be filled with text.
//!
//! A [`TextInput`] has some local [`State`].
//!
//! [`TextInput`]: struct.TextInput.html
//! [`State`]: struct.State.html
use crate::Length;

/// A widget that can be filled with text by using a keyboard.
#[allow(missing_docs)]
pub struct TextInput<'a, Message> {
    pub state: &'a mut State,
    pub placeholder: String,
    pub value: Value,
    pub width: Length,
    pub max_width: Length,
    pub padding: u16,
    pub size: Option<u16>,
    pub on_change: Box<dyn Fn(String) -> Message>,
    pub on_submit: Option<Message>,
}

impl<'a, Message> TextInput<'a, Message> {
    /// Creates a new [`TextInput`].
    ///
    /// [`TextInput`]: struct.TextInput.html
    pub fn new<F>(
        state: &'a mut State,
        placeholder: &str,
        value: &str,
        on_change: F,
    ) -> Self
    where
        F: 'static + Fn(String) -> Message,
    {
        Self {
            state,
            placeholder: String::from(placeholder),
            value: Value::new(value),
            width: Length::Fill,
            max_width: Length::Shrink,
            padding: 0,
            size: None,
            on_change: Box::new(on_change),
            on_submit: None,
        }
    }

    /// Sets the width of the [`TextInput`].
    ///
    /// [`TextInput`]: struct.TextInput.html
    pub fn width(mut self, width: Length) -> Self {
        self.width = width;
        self
    }

    /// Sets the maximum width of the [`TextInput`].
    ///
    /// [`TextInput`]: struct.TextInput.html
    pub fn max_width(mut self, max_width: Length) -> Self {
        self.max_width = max_width;
        self
    }

    /// Sets the padding of the [`TextInput`].
    ///
    /// [`TextInput`]: struct.TextInput.html
    pub fn padding(mut self, units: u16) -> Self {
        self.padding = units;
        self
    }

    /// Sets the text size of the [`TextInput`].
    ///
    /// [`TextInput`]: struct.TextInput.html
    pub fn size(mut self, size: u16) -> Self {
        self.size = Some(size);
        self
    }

    /// Sets the message that should be produced when the [`TextInput`] is
    /// focused and the enter key is pressed.
    ///
    /// [`TextInput`]: struct.TextInput.html
    pub fn on_submit(mut self, message: Message) -> Self {
        self.on_submit = Some(message);
        self
    }
}

impl<'a, Message> std::fmt::Debug for TextInput<'a, Message>
where
    Message: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // TODO: Complete once stabilized
        f.debug_struct("TextInput").finish()
    }
}

/// The state of a [`TextInput`].
///
/// [`TextInput`]: struct.TextInput.html
#[derive(Debug, Default, Clone)]
pub struct State {
    /// Whether the [`TextInput`] is focused or not.
    ///
    /// [`TextInput`]: struct.TextInput.html
    pub is_focused: bool,
    cursor_position: usize,
}

impl State {
    /// Creates a new [`State`], representing an unfocused [`TextInput`].
    ///
    /// [`State`]: struct.State.html
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a new [`State`], representing a focused [`TextInput`].
    ///
    /// [`State`]: struct.State.html
    pub fn focused() -> Self {
        use std::usize;

        Self {
            is_focused: true,
            cursor_position: usize::MAX,
        }
    }

    /// Moves the cursor of a [`TextInput`] to the right.
    ///
    /// [`TextInput`]: struct.TextInput.html
    pub fn move_cursor_right(&mut self, value: &Value) {
        let current = self.cursor_position(value);

        if current < value.len() {
            self.cursor_position = current + 1;
        }
    }

    /// Moves the cursor of a [`TextInput`] to the left.
    ///
    /// [`TextInput`]: struct.TextInput.html
    pub fn move_cursor_left(&mut self, value: &Value) {
        let current = self.cursor_position(value);

        if current > 0 {
            self.cursor_position = current - 1;
        }
    }

    /// Moves the cursor of a [`TextInput`] to the end.
    ///
    /// [`TextInput`]: struct.TextInput.html
    pub fn move_cursor_to_end(&mut self, value: &Value) {
        self.cursor_position = value.len();
    }

    /// Returns the cursor position of a [`TextInput`].
    ///
    /// [`TextInput`]: struct.TextInput.html
    pub fn cursor_position(&self, value: &Value) -> usize {
        self.cursor_position.min(value.len())
    }
}

/// The value of a [`TextInput`].
///
/// [`TextInput`]: struct.TextInput.html
// TODO: Use `unicode-segmentation`
#[derive(Debug)]
pub struct Value(Vec<char>);

impl Value {
    /// Creates a new [`Value`] from a string slice.
    ///
    /// [`Value`]: struct.Value.html
    pub fn new(string: &str) -> Self {
        Self(string.chars().collect())
    }

    /// Returns the total amount of `char` in the [`Value`].
    ///
    /// [`Value`]: struct.Value.html
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns a new [`Value`] containing the `char` until the given `index`.
    ///
    /// [`Value`]: struct.Value.html
    pub fn until(&self, index: usize) -> Self {
        Self(self.0[..index.min(self.len())].to_vec())
    }

    /// Converts the [`Value`] into a `String`.
    ///
    /// [`Value`]: struct.Value.html
    pub fn to_string(&self) -> String {
        use std::iter::FromIterator;
        String::from_iter(self.0.iter())
    }

    /// Inserts a new `char` at the given `index`.
    ///
    /// [`Value`]: struct.Value.html
    pub fn insert(&mut self, index: usize, c: char) {
        self.0.insert(index, c);
    }

    /// Removes the `char` at the given `index`.
    ///
    /// [`Value`]: struct.Value.html
    pub fn remove(&mut self, index: usize) {
        let _ = self.0.remove(index);
    }
}
