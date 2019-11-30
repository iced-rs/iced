//! Display fields that can be filled with text.
//!
//! A [`TextInput`] has some local [`State`].
//!
//! [`TextInput`]: struct.TextInput.html
//! [`State`]: struct.State.html
use crate::{
    input::{keyboard, mouse, ButtonState},
    layout, Element, Event, Hasher, Layout, Length, Point, Rectangle, Size,
    Widget,
};

/// A field that can be filled with text.
///
/// # Example
/// ```
/// # use iced_native::{text_input, TextInput};
/// #
/// #[derive(Debug, Clone)]
/// enum Message {
///     TextInputChanged(String),
/// }
///
/// let mut state = text_input::State::new();
/// let value = "Some text";
///
/// let input = TextInput::new(
///     &mut state,
///     "This is the placeholder...",
///     value,
///     Message::TextInputChanged,
/// )
/// .padding(10);
/// ```
/// ![Text input drawn by `iced_wgpu`](https://github.com/hecrj/iced/blob/7760618fb112074bc40b148944521f312152012a/docs/images/text_input.png?raw=true)
#[allow(missing_debug_implementations)]
pub struct TextInput<'a, Message> {
    state: &'a mut State,
    placeholder: String,
    value: Value,
    width: Length,
    max_width: Length,
    padding: u16,
    size: Option<u16>,
    on_change: Box<dyn Fn(String) -> Message>,
    on_submit: Option<Message>,
}

impl<'a, Message> TextInput<'a, Message> {
    /// Creates a new [`TextInput`].
    ///
    /// It expects:
    /// - some [`State`]
    /// - a placeholder
    /// - the current value
    /// - a function that produces a message when the [`TextInput`] changes
    ///
    /// [`TextInput`]: struct.TextInput.html
    /// [`State`]: struct.State.html
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

impl<'a, Message, Renderer> Widget<Message, Renderer> for TextInput<'a, Message>
where
    Renderer: self::Renderer,
    Message: Clone + std::fmt::Debug,
{
    fn width(&self) -> Length {
        self.width
    }

    fn height(&self) -> Length {
        Length::Shrink
    }

    fn layout(
        &self,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let padding = self.padding as f32;
        let text_size = self.size.unwrap_or(renderer.default_size());

        let limits = limits
            .pad(padding)
            .width(self.width)
            .height(Length::Units(text_size));

        let mut text = layout::Node::new(limits.resolve(Size::ZERO));
        text.bounds.x = padding;
        text.bounds.y = padding;

        layout::Node::with_children(text.size().pad(padding), vec![text])
    }

    fn on_event(
        &mut self,
        event: Event,
        layout: Layout<'_>,
        cursor_position: Point,
        messages: &mut Vec<Message>,
        _renderer: &Renderer,
    ) {
        match event {
            Event::Mouse(mouse::Event::Input {
                button: mouse::Button::Left,
                state: ButtonState::Pressed,
            }) => {
                self.state.is_focused =
                    layout.bounds().contains(cursor_position);

                if self.state.cursor_position(&self.value) == 0 {
                    self.state.move_cursor_to_end(&self.value);
                }
            }
            Event::Keyboard(keyboard::Event::CharacterReceived(c))
                if self.state.is_focused && !c.is_control() =>
            {
                let cursor_position = self.state.cursor_position(&self.value);

                self.value.insert(cursor_position, c);
                self.state.move_cursor_right(&self.value);

                let message = (self.on_change)(self.value.to_string());
                messages.push(message);
            }
            Event::Keyboard(keyboard::Event::Input {
                key_code,
                state: ButtonState::Pressed,
            }) if self.state.is_focused => match key_code {
                keyboard::KeyCode::Enter => {
                    if let Some(on_submit) = self.on_submit.clone() {
                        messages.push(on_submit);
                    }
                }
                keyboard::KeyCode::Backspace => {
                    let cursor_position =
                        self.state.cursor_position(&self.value);

                    if cursor_position > 0 {
                        self.state.move_cursor_left(&self.value);

                        let _ = self.value.remove(cursor_position - 1);

                        let message = (self.on_change)(self.value.to_string());
                        messages.push(message);
                    }
                }
                keyboard::KeyCode::Delete => {
                    let cursor_position =
                        self.state.cursor_position(&self.value);

                    if cursor_position < self.value.len() {
                        let _ = self.value.remove(cursor_position);

                        let message = (self.on_change)(self.value.to_string());
                        messages.push(message);
                    }
                }
                keyboard::KeyCode::Left => {
                    self.state.move_cursor_left(&self.value);
                }
                keyboard::KeyCode::Right => {
                    self.state.move_cursor_right(&self.value);
                }
                _ => {}
            },
            _ => {}
        }
    }

    fn draw(
        &self,
        renderer: &mut Renderer,
        layout: Layout<'_>,
        cursor_position: Point,
    ) -> Renderer::Output {
        let bounds = layout.bounds();
        let text_bounds = layout.children().next().unwrap().bounds();

        renderer.draw(
            bounds,
            text_bounds,
            cursor_position,
            self.size.unwrap_or(renderer.default_size()),
            &self.placeholder,
            &self.value,
            &self.state,
        )
    }

    fn hash_layout(&self, state: &mut Hasher) {
        use std::{any::TypeId, hash::Hash};

        TypeId::of::<TextInput<'static, ()>>().hash(state);

        self.width.hash(state);
        self.max_width.hash(state);
        self.padding.hash(state);
        self.size.hash(state);
    }
}

/// The renderer of a [`TextInput`].
///
/// Your [renderer] will need to implement this trait before being
/// able to use a [`TextInput`] in your user interface.
///
/// [`TextInput`]: struct.TextInput.html
/// [renderer]: ../../renderer/index.html
pub trait Renderer: crate::Renderer + Sized {
    /// Returns the default size of the text of the [`TextInput`].
    ///
    /// [`TextInput`]: struct.TextInput.html
    fn default_size(&self) -> u16;

    /// Draws a [`TextInput`].
    ///
    /// It receives:
    /// - its bounds of the [`TextInput`]
    /// - the bounds of the text (i.e. the current value)
    /// - the cursor position
    /// - the placeholder to show when the value is empty
    /// - the current [`Value`]
    /// - the current [`State`]
    ///
    /// [`TextInput`]: struct.TextInput.html
    /// [`Value`]: struct.Value.html
    /// [`State`]: struct.State.html
    fn draw(
        &mut self,
        bounds: Rectangle,
        text_bounds: Rectangle,
        cursor_position: Point,
        size: u16,
        placeholder: &str,
        value: &Value,
        state: &State,
    ) -> Self::Output;
}

impl<'a, Message, Renderer> From<TextInput<'a, Message>>
    for Element<'a, Message, Renderer>
where
    Renderer: 'static + self::Renderer,
    Message: 'static + Clone + std::fmt::Debug,
{
    fn from(
        text_input: TextInput<'a, Message>,
    ) -> Element<'a, Message, Renderer> {
        Element::new(text_input)
    }
}

/// The state of a [`TextInput`].
///
/// [`TextInput`]: struct.TextInput.html
#[derive(Debug, Default, Clone)]
pub struct State {
    is_focused: bool,
    cursor_position: usize,
}

impl State {
    /// Creates a new [`State`], representing an unfocused [`TextInput`].
    ///
    /// [`State`]: struct.State.html
    pub const fn new() -> Self {
        Self {
            is_focused: false,
            cursor_position: 0,
        }
    }

    /// Creates a new [`State`], representing a focused [`TextInput`].
    ///
    /// [`State`]: struct.State.html
    pub const fn focused() -> Self {
        use std::usize;

        Self {
            is_focused: true,
            cursor_position: usize::MAX,
        }
    }

    /// Returns whether the [`TextInput`] is currently focused or not.
    ///
    /// [`TextInput`]: struct.TextInput.html
    pub const fn is_focused(&self) -> bool {
        self.is_focused
    }

    /// Returns the cursor position of a [`TextInput`].
    ///
    /// [`TextInput`]: struct.TextInput.html
    pub fn cursor_position(&self, value: &Value) -> usize {
        self.cursor_position.min(value.len())
    }

    /// Moves the cursor of a [`TextInput`] to the right.
    ///
    /// [`TextInput`]: struct.TextInput.html
    pub(crate) fn move_cursor_right(&mut self, value: &Value) {
        let current = self.cursor_position(value);

        if current < value.len() {
            self.cursor_position = current + 1;
        }
    }

    /// Moves the cursor of a [`TextInput`] to the left.
    ///
    /// [`TextInput`]: struct.TextInput.html
    pub(crate) fn move_cursor_left(&mut self, value: &Value) {
        let current = self.cursor_position(value);

        if current > 0 {
            self.cursor_position = current - 1;
        }
    }

    /// Moves the cursor of a [`TextInput`] to the end.
    ///
    /// [`TextInput`]: struct.TextInput.html
    pub(crate) fn move_cursor_to_end(&mut self, value: &Value) {
        self.cursor_position = value.len();
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
