//! Display fields that can be filled with text.
//!
//! A [`TextInput`] has some local [`State`].
//!
//! [`TextInput`]: struct.TextInput.html
//! [`State`]: struct.State.html
use crate::{
    input::{keyboard, mouse, ButtonState},
    layout, Clipboard, Element, Event, Font, Hasher, Layout, Length, Point,
    Rectangle, Size, Widget,
};

use std::u32;
use unicode_segmentation::UnicodeSegmentation;

/// A field that can be filled with text.
///
/// # Example
/// ```
/// # use iced_native::{text_input, renderer::Null};
/// #
/// # pub type TextInput<'a, Message> = iced_native::TextInput<'a, Message, Null>;
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
pub struct TextInput<'a, Message, Renderer: self::Renderer> {
    state: &'a mut State,
    placeholder: String,
    value: Value,
    is_secure: bool,
    font: Font,
    width: Length,
    max_width: u32,
    padding: u16,
    size: Option<u16>,
    on_change: Box<dyn Fn(String) -> Message>,
    on_submit: Option<Message>,
    style: Renderer::Style,
}

impl<'a, Message, Renderer: self::Renderer> TextInput<'a, Message, Renderer> {
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
        TextInput {
            state,
            placeholder: String::from(placeholder),
            value: Value::new(value),
            is_secure: false,
            font: Font::Default,
            width: Length::Fill,
            max_width: u32::MAX,
            padding: 0,
            size: None,
            on_change: Box::new(on_change),
            on_submit: None,
            style: Renderer::Style::default(),
        }
    }

    /// Converts the [`TextInput`] into a secure password input.
    ///
    /// [`TextInput`]: struct.TextInput.html
    pub fn password(mut self) -> Self {
        self.is_secure = true;
        self
    }

    /// Sets the [`Font`] of the [`Text`].
    ///
    /// [`Text`]: struct.Text.html
    /// [`Font`]: ../../struct.Font.html
    pub fn font(mut self, font: Font) -> Self {
        self.font = font;
        self
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
    pub fn max_width(mut self, max_width: u32) -> Self {
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

    /// Sets the style of the [`TextInput`].
    ///
    /// [`TextInput`]: struct.TextInput.html
    pub fn style(mut self, style: impl Into<Renderer::Style>) -> Self {
        self.style = style.into();
        self
    }
}

impl<'a, Message, Renderer> Widget<Message, Renderer>
    for TextInput<'a, Message, Renderer>
where
    Renderer: 'static + self::Renderer,
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
            .max_width(self.max_width)
            .height(Length::Units(text_size));

        let mut text = layout::Node::new(limits.resolve(Size::ZERO));
        text.move_to(Point::new(padding, padding));

        layout::Node::with_children(text.size().pad(padding), vec![text])
    }

    fn on_event(
        &mut self,
        event: Event,
        layout: Layout<'_>,
        cursor_position: Point,
        messages: &mut Vec<Message>,
        renderer: &Renderer,
        clipboard: Option<&dyn Clipboard>,
    ) {
        match event {
            Event::Mouse(mouse::Event::Input {
                button: mouse::Button::Left,
                state: ButtonState::Pressed,
            }) => {
                let is_clicked = layout.bounds().contains(cursor_position);

                if is_clicked {
                    let text_layout = layout.children().next().unwrap();
                    let target = cursor_position.x - text_layout.bounds().x;

                    if target > 0.0 {
                        let value = if self.is_secure {
                            self.value.secure()
                        } else {
                            self.value.clone()
                        };

                        let size = self.size.unwrap_or(renderer.default_size());

                        let offset = renderer.offset(
                            text_layout.bounds(),
                            size,
                            &value,
                            &self.state,
                            self.font,
                        );

                        self.state.cursor_position =
                            Cursor::Index(find_cursor_position(
                                renderer,
                                target + offset,
                                &value,
                                size,
                                0,
                                self.value.len(),
                                self.font,
                            ));
                    } else {
                        self.state.cursor_position = Cursor::Index(0);
                    }
                }

                self.state.is_pressed = is_clicked;
                self.state.is_focused = is_clicked;
            }
            Event::Mouse(mouse::Event::Input {
                button: mouse::Button::Left,
                state: ButtonState::Released,
            }) => {
                self.state.is_pressed = false;
            }
            Event::Mouse(mouse::Event::CursorMoved { x, .. }) => {
                if self.state.is_pressed {
                    let text_layout = layout.children().next().unwrap();
                    let target = x - text_layout.bounds().x;

                    if target > 0.0 {
                        let value = if self.is_secure {
                            self.value.secure()
                        } else {
                            self.value.clone()
                        };

                        let size = self.size.unwrap_or(renderer.default_size());

                        let offset = renderer.offset(
                            text_layout.bounds(),
                            size,
                            &value,
                            &self.state,
                            self.font,
                        );

                        let pos = find_cursor_position(
                            renderer,
                            target + offset,
                            &value,
                            size,
                            0,
                            self.value.len(),
                            self.font,
                        );

                        let start = match self.state.cursor_position {
                            Cursor::Index(index) => index,
                            Cursor::Selection { start, .. } => start,
                        };

                        self.state.cursor_position =
                            Cursor::Selection { start, end: pos }.simplify();
                    }
                }
            }
            Event::Keyboard(keyboard::Event::CharacterReceived(c))
                if self.state.is_focused
                    && self.state.is_pasting.is_none()
                    && !c.is_control() =>
            {
                let cursor = self.state.cursor_position(&self.value);

                match cursor {
                    Cursor::Index(index) => {
                        self.value.insert(index, c);
                    }
                    Cursor::Selection { .. } => {
                        self.state.move_cursor_left(&self.value);
                        self.value.remove_many(cursor.left(), cursor.right());
                        self.value.insert(cursor.left(), c);
                    }
                }
                self.state.move_cursor_right(&self.value);

                let message = (self.on_change)(self.value.to_string());
                messages.push(message);
            }
            Event::Keyboard(keyboard::Event::Input {
                key_code,
                state: ButtonState::Pressed,
                modifiers,
            }) if self.state.is_focused => match key_code {
                keyboard::KeyCode::Enter => {
                    if let Some(on_submit) = self.on_submit.clone() {
                        messages.push(on_submit);
                    }
                }
                keyboard::KeyCode::Backspace => {
                    let cursor = self.state.cursor_position(&self.value);

                    match cursor {
                        Cursor::Index(index) if index > 0 => {
                            self.state.move_cursor_left(&self.value);
                            let _ = self.value.remove(index - 1);
                            let message =
                                (self.on_change)(self.value.to_string());
                            messages.push(message);
                        }
                        Cursor::Selection { .. } => {
                            self.state.move_cursor_left(&self.value);
                            self.value
                                .remove_many(cursor.left(), cursor.right());
                            let message =
                                (self.on_change)(self.value.to_string());
                            messages.push(message);
                        }
                        _ => {}
                    }
                }
                keyboard::KeyCode::Delete => {
                    let cursor = self.state.cursor_position(&self.value);

                    match cursor {
                        Cursor::Index(index) if index < self.value.len() => {
                            let _ = self.value.remove(index);
                            let message =
                                (self.on_change)(self.value.to_string());
                            messages.push(message);
                        }
                        Cursor::Selection { .. } => {
                            self.state.move_cursor_left(&self.value);
                            self.value
                                .remove_many(cursor.left(), cursor.right());
                            let message =
                                (self.on_change)(self.value.to_string());
                            messages.push(message);
                        }
                        _ => {}
                    }
                }
                keyboard::KeyCode::Left => {
                    if platform::is_jump_modifier_pressed(modifiers)
                        && !self.is_secure
                    {
                        self.state.move_cursor_left_by_words(&self.value);
                    } else {
                        self.state.move_cursor_left(&self.value);
                    }
                }
                keyboard::KeyCode::Right => {
                    if platform::is_jump_modifier_pressed(modifiers)
                        && !self.is_secure
                    {
                        self.state.move_cursor_right_by_words(&self.value);
                    } else {
                        self.state.move_cursor_right(&self.value);
                    }
                }
                keyboard::KeyCode::Home => {
                    self.state.cursor_position = Cursor::Index(0);
                }
                keyboard::KeyCode::End => {
                    self.state.move_cursor_to_end(&self.value);
                }
                keyboard::KeyCode::V => {
                    if platform::is_copy_paste_modifier_pressed(modifiers) {
                        if let Some(clipboard) = clipboard {
                            let content = match self.state.is_pasting.take() {
                                Some(content) => content,
                                None => {
                                    let content: String = clipboard
                                        .content()
                                        .unwrap_or(String::new())
                                        .chars()
                                        .filter(|c| !c.is_control())
                                        .collect();

                                    Value::new(&content)
                                }
                            };

                            let cursor_position =
                                self.state.cursor_position(&self.value);

                            let insert_position = match cursor_position {
                                Cursor::Index(index) => index,
                                Cursor::Selection { .. } => {
                                    self.state.move_cursor_left(&self.value);
                                    self.value.remove_many(
                                        cursor_position.left(),
                                        cursor_position.right(),
                                    );
                                    cursor_position.left()
                                }
                            };
                            self.value
                                .insert_many(insert_position, content.clone());

                            self.state.move_cursor_right_by_amount(
                                &self.value,
                                content.len(),
                            );
                            self.state.is_pasting = Some(content);

                            let message =
                                (self.on_change)(self.value.to_string());
                            messages.push(message);
                        }
                    } else {
                        self.state.is_pasting = None;
                    }
                }
                keyboard::KeyCode::A => {
                    if platform::is_copy_paste_modifier_pressed(modifiers) {
                        self.state.cursor_position = {
                            Cursor::Selection {
                                start: 0,
                                end: self.value.len(),
                            }
                        }
                    }
                }
                _ => {}
            },
            Event::Keyboard(keyboard::Event::Input {
                key_code,
                state: ButtonState::Released,
                ..
            }) => match key_code {
                keyboard::KeyCode::V => {
                    self.state.is_pasting = None;
                }
                _ => {}
            },
            _ => {}
        }
    }

    fn draw(
        &self,
        renderer: &mut Renderer,
        _defaults: &Renderer::Defaults,
        layout: Layout<'_>,
        cursor_position: Point,
    ) -> Renderer::Output {
        let bounds = layout.bounds();
        let text_bounds = layout.children().next().unwrap().bounds();

        if self.is_secure {
            renderer.draw(
                bounds,
                text_bounds,
                cursor_position,
                self.size.unwrap_or(renderer.default_size()),
                self.font,
                &self.placeholder,
                &self.value.secure(),
                &self.state,
                &self.style,
            )
        } else {
            renderer.draw(
                bounds,
                text_bounds,
                cursor_position,
                self.size.unwrap_or(renderer.default_size()),
                self.font,
                &self.placeholder,
                &self.value,
                &self.state,
                &self.style,
            )
        }
    }

    fn hash_layout(&self, state: &mut Hasher) {
        use std::{any::TypeId, hash::Hash};

        TypeId::of::<TextInput<'static, (), Renderer>>().hash(state);

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
    /// The style supported by this renderer.
    type Style: Default;

    /// Returns the default size of the text of the [`TextInput`].
    ///
    /// [`TextInput`]: struct.TextInput.html
    fn default_size(&self) -> u16;

    /// Returns the width of the value of the [`TextInput`].
    ///
    /// [`TextInput`]: struct.TextInput.html
    fn measure_value(&self, value: &str, size: u16, font: Font) -> f32;

    /// Returns the current horizontal offset of the value of the
    /// [`TextInput`].
    ///
    /// This is the amount of horizontal scrolling applied when the [`Value`]
    /// does not fit the [`TextInput`].
    ///
    /// [`TextInput`]: struct.TextInput.html
    /// [`Value`]: struct.Value.html
    fn offset(
        &self,
        text_bounds: Rectangle,
        size: u16,
        value: &Value,
        state: &State,
        font: Font,
    ) -> f32;

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
        font: Font,
        placeholder: &str,
        value: &Value,
        state: &State,
        style: &Self::Style,
    ) -> Self::Output;
}

impl<'a, Message, Renderer> From<TextInput<'a, Message, Renderer>>
    for Element<'a, Message, Renderer>
where
    Renderer: 'static + self::Renderer,
    Message: 'static + Clone + std::fmt::Debug,
{
    fn from(
        text_input: TextInput<'a, Message, Renderer>,
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
    is_pressed: bool,
    is_pasting: Option<Value>,
    cursor_position: Cursor,
    // TODO: Add stateful horizontal scrolling offset
}

/// The cursor position of a [`TextInput`].
///
/// [`TextInput`]: struct.TextInput.html
#[derive(Debug, Clone, Copy)]
pub enum Cursor {
    /// The cursor represents a position.
    Index(usize),
    /// The cursor represents a range.
    Selection {
        /// Where the selection started.
        start: usize,
        /// Where the selection was moved to.
        end: usize,
    },
}

impl Default for Cursor {
    fn default() -> Self {
        Cursor::Index(0)
    }
}

impl Cursor {
    /// Simplify representation to `Cursor::Index`
    /// if `start` and `end` are the same.
    pub fn simplify(&self) -> Self {
        match self {
            Cursor::Index(_) => *self,
            Cursor::Selection { start, end } => {
                if start == end {
                    Cursor::Index(*start)
                } else {
                    *self
                }
            }
        }
    }

    /// Position at which the cursor should be drawn.
    pub fn position(&self) -> usize {
        match *self {
            Cursor::Index(index) => index,
            Cursor::Selection { end, .. } => end,
        }
    }

    /// The cursor index or left end of the selection.
    pub fn left(&self) -> usize {
        match *self {
            Cursor::Index(index) => index,
            Cursor::Selection { start, end } => start.min(end),
        }
    }

    /// The cursor index or right end of the selection.
    pub fn right(&self) -> usize {
        match *self {
            Cursor::Index(index) => index,
            Cursor::Selection { start, end } => start.max(end),
        }
    }
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
            is_pressed: false,
            is_pasting: None,
            cursor_position: Cursor::Index(usize::MAX),
        }
    }

    /// Returns whether the [`TextInput`] is currently focused or not.
    ///
    /// [`TextInput`]: struct.TextInput.html
    pub fn is_focused(&self) -> bool {
        self.is_focused
    }

    /// Returns the cursor position of a [`TextInput`].
    ///
    /// [`TextInput`]: struct.TextInput.html
    pub fn cursor_position(&self, value: &Value) -> Cursor {
        match self.cursor_position {
            Cursor::Index(index) => Cursor::Index(index.min(value.len())),
            Cursor::Selection { start, end } => Cursor::Selection {
                start: start.min(value.len()),
                end: end.min(value.len()),
            }
            .simplify(),
        }
    }

    /// Moves the cursor of a [`TextInput`] to the left.
    ///
    /// [`TextInput`]: struct.TextInput.html
    pub(crate) fn move_cursor_left(&mut self, value: &Value) {
        let current = self.cursor_position(value);

        self.cursor_position = match current {
            Cursor::Index(index) if index > 0 => Cursor::Index(index - 1),
            Cursor::Selection { .. } => Cursor::Index(current.left()),
            _ => Cursor::Index(0),
        }
    }

    /// Moves the cursor of a [`TextInput`] to the right.
    ///
    /// [`TextInput`]: struct.TextInput.html
    pub(crate) fn move_cursor_right(&mut self, value: &Value) {
        self.move_cursor_right_by_amount(value, 1)
    }

    pub(crate) fn move_cursor_right_by_amount(
        &mut self,
        value: &Value,
        amount: usize,
    ) {
        let current = self.cursor_position(value);
        self.cursor_position = match current {
            Cursor::Index(index) => {
                Cursor::Index(index.saturating_add(amount).min(value.len()))
            }
            Cursor::Selection { .. } => Cursor::Index(current.right()),
        };
    }

    /// Moves the cursor of a [`TextInput`] to the previous start of a word.
    ///
    /// [`TextInput`]: struct.TextInput.html
    pub(crate) fn move_cursor_left_by_words(&mut self, value: &Value) {
        let current = self.cursor_position(value);

        self.cursor_position =
            Cursor::Index(value.previous_start_of_word(current.left()));
    }

    /// Moves the cursor of a [`TextInput`] to the next end of a word.
    ///
    /// [`TextInput`]: struct.TextInput.html
    pub(crate) fn move_cursor_right_by_words(&mut self, value: &Value) {
        let current = self.cursor_position(value);

        self.cursor_position =
            Cursor::Index(value.next_end_of_word(current.right()));
    }

    /// Moves the cursor of a [`TextInput`] to the end.
    ///
    /// [`TextInput`]: struct.TextInput.html
    pub(crate) fn move_cursor_to_end(&mut self, value: &Value) {
        self.cursor_position = Cursor::Index(value.len());
    }
}

/// The value of a [`TextInput`].
///
/// [`TextInput`]: struct.TextInput.html
// TODO: Reduce allocations, cache results (?)
#[derive(Debug, Clone)]
pub struct Value {
    graphemes: Vec<String>,
}

impl Value {
    /// Creates a new [`Value`] from a string slice.
    ///
    /// [`Value`]: struct.Value.html
    pub fn new(string: &str) -> Self {
        let graphemes = UnicodeSegmentation::graphemes(string, true)
            .map(String::from)
            .collect();

        Self { graphemes }
    }

    /// Returns the total amount of graphemes in the [`Value`].
    ///
    /// [`Value`]: struct.Value.html
    pub fn len(&self) -> usize {
        self.graphemes.len()
    }

    /// Returns the position of the previous start of a word from the given
    /// grapheme `index`.
    ///
    /// [`Value`]: struct.Value.html
    pub fn previous_start_of_word(&self, index: usize) -> usize {
        let previous_string =
            &self.graphemes[..index.min(self.graphemes.len())].concat();

        UnicodeSegmentation::split_word_bound_indices(&previous_string as &str)
            .filter(|(_, word)| !word.trim_start().is_empty())
            .next_back()
            .map(|(i, previous_word)| {
                index
                    - UnicodeSegmentation::graphemes(previous_word, true)
                        .count()
                    - UnicodeSegmentation::graphemes(
                        &previous_string[i + previous_word.len()..] as &str,
                        true,
                    )
                    .count()
            })
            .unwrap_or(0)
    }

    /// Returns the position of the next end of a word from the given grapheme
    /// `index`.
    ///
    /// [`Value`]: struct.Value.html
    pub fn next_end_of_word(&self, index: usize) -> usize {
        let next_string = &self.graphemes[index..].concat();

        UnicodeSegmentation::split_word_bound_indices(&next_string as &str)
            .filter(|(_, word)| !word.trim_start().is_empty())
            .next()
            .map(|(i, next_word)| {
                index
                    + UnicodeSegmentation::graphemes(next_word, true).count()
                    + UnicodeSegmentation::graphemes(
                        &next_string[..i] as &str,
                        true,
                    )
                    .count()
            })
            .unwrap_or(self.len())
    }

    /// Returns a new [`Value`] containing the graphemes until the given
    /// `index`.
    ///
    /// [`Value`]: struct.Value.html
    pub fn until(&self, index: usize) -> Self {
        let graphemes = self.graphemes[..index.min(self.len())].to_vec();

        Self { graphemes }
    }

    /// Converts the [`Value`] into a `String`.
    ///
    /// [`Value`]: struct.Value.html
    pub fn to_string(&self) -> String {
        self.graphemes.concat()
    }

    /// Inserts a new `char` at the given grapheme `index`.
    pub fn insert(&mut self, index: usize, c: char) {
        self.graphemes.insert(index, c.to_string());

        self.graphemes =
            UnicodeSegmentation::graphemes(&self.to_string() as &str, true)
                .map(String::from)
                .collect();
    }

    /// Inserts a bunch of graphemes at the given grapheme `index`.
    pub fn insert_many(&mut self, index: usize, mut value: Value) {
        let _ = self
            .graphemes
            .splice(index..index, value.graphemes.drain(..));
    }

    /// Removes the grapheme at the given `index`.
    ///
    /// [`Value`]: struct.Value.html
    pub fn remove(&mut self, index: usize) {
        let _ = self.graphemes.remove(index);
    }

    /// Removes the graphemes from `start` to `end`.
    pub fn remove_many(&mut self, start: usize, end: usize) {
        let _ = self.graphemes.splice(start..end, std::iter::empty());
    }

    /// Returns a new [`Value`] with all its graphemes replaced with the
    /// dot ('•') character.
    ///
    /// [`Value`]: struct.Value.html
    pub fn secure(&self) -> Self {
        Self {
            graphemes: std::iter::repeat(String::from("•"))
                .take(self.graphemes.len())
                .collect(),
        }
    }
}

// TODO: Reduce allocations
fn find_cursor_position<Renderer: self::Renderer>(
    renderer: &Renderer,
    target: f32,
    value: &Value,
    size: u16,
    start: usize,
    end: usize,
    font: Font,
) -> usize {
    if start >= end {
        if start == 0 {
            return 0;
        }

        let prev = value.until(start - 1);
        let next = value.until(start);

        let prev_width = renderer.measure_value(&prev.to_string(), size, font);
        let next_width = renderer.measure_value(&next.to_string(), size, font);

        if next_width - target > target - prev_width {
            return start - 1;
        } else {
            return start;
        }
    }

    let index = (end - start) / 2;
    let subvalue = value.until(start + index);

    let width = renderer.measure_value(&subvalue.to_string(), size, font);

    if width > target {
        find_cursor_position(
            renderer,
            target,
            value,
            size,
            start,
            start + index,
            font,
        )
    } else {
        find_cursor_position(
            renderer,
            target,
            value,
            size,
            start + index + 1,
            end,
            font,
        )
    }
}

mod platform {
    use crate::input::keyboard;

    pub fn is_jump_modifier_pressed(
        modifiers: keyboard::ModifiersState,
    ) -> bool {
        if cfg!(target_os = "macos") {
            modifiers.alt
        } else {
            modifiers.control
        }
    }

    pub fn is_copy_paste_modifier_pressed(
        modifiers: keyboard::ModifiersState,
    ) -> bool {
        if cfg!(target_os = "macos") {
            modifiers.logo
        } else {
            modifiers.control
        }
    }
}
