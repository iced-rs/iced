use crate::Length;

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

    pub fn size(mut self, size: u16) -> Self {
        self.size = Some(size);
        self
    }

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

#[derive(Debug, Default)]
pub struct State {
    pub is_focused: bool,
    cursor_position: usize,
}

impl State {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn move_cursor_right(&mut self, value: &Value) {
        let current = self.cursor_position(value);

        if current < value.len() {
            self.cursor_position = current + 1;
        }
    }

    pub fn move_cursor_left(&mut self, value: &Value) {
        let current = self.cursor_position(value);

        if current > 0 {
            self.cursor_position = current - 1;
        }
    }

    pub fn cursor_position(&self, value: &Value) -> usize {
        self.cursor_position.min(value.len())
    }
}

// TODO: Use `unicode-segmentation`
pub struct Value(Vec<char>);

impl Value {
    pub fn new(string: &str) -> Self {
        Self(string.chars().collect())
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn until(&self, index: usize) -> Self {
        Self(self.0[..index.min(self.len())].to_vec())
    }

    pub fn to_string(&self) -> String {
        use std::iter::FromIterator;
        String::from_iter(self.0.iter())
    }

    pub fn insert(&mut self, index: usize, c: char) {
        self.0.insert(index, c);
    }

    pub fn remove(&mut self, index: usize) {
        self.0.remove(index);
    }
}
