//! Display fields that can be filled with text.
//!
//! A [`TextInput`] has some local [`State`].
mod editor;
mod value;

pub mod cursor;

pub use cursor::Cursor;
pub use value::Value;

use editor::Editor;

use crate::event::{self, Event};
use crate::keyboard;
use crate::layout;
use crate::mouse::{self, click};
use crate::text;
use crate::touch;
use crate::{
    Clipboard, Element, Hasher, Layout, Length, Padding, Point, Rectangle,
    Size, Widget,
};

use std::u32;

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
    font: Renderer::Font,
    width: Length,
    max_width: u32,
    padding: Padding,
    size: Option<u16>,
    on_change: Box<dyn Fn(String) -> Message>,
    on_submit: Option<Message>,
    style: Renderer::Style,
}

impl<'a, Message, Renderer> TextInput<'a, Message, Renderer>
where
    Message: Clone,
    Renderer: self::Renderer,
{
    /// Creates a new [`TextInput`].
    ///
    /// It expects:
    /// - some [`State`]
    /// - a placeholder
    /// - the current value
    /// - a function that produces a message when the [`TextInput`] changes
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
            font: Default::default(),
            width: Length::Fill,
            max_width: u32::MAX,
            padding: Padding::ZERO,
            size: None,
            on_change: Box::new(on_change),
            on_submit: None,
            style: Renderer::Style::default(),
        }
    }

    /// Converts the [`TextInput`] into a secure password input.
    pub fn password(mut self) -> Self {
        self.is_secure = true;
        self
    }

    /// Sets the [`Font`] of the [`Text`].
    ///
    /// [`Font`]: crate::widget::text::Renderer::Font
    /// [`Text`]: crate::widget::Text
    pub fn font(mut self, font: Renderer::Font) -> Self {
        self.font = font;
        self
    }
    /// Sets the width of the [`TextInput`].
    pub fn width(mut self, width: Length) -> Self {
        self.width = width;
        self
    }

    /// Sets the maximum width of the [`TextInput`].
    pub fn max_width(mut self, max_width: u32) -> Self {
        self.max_width = max_width;
        self
    }

    /// Sets the [`Padding`] of the [`TextInput`].
    pub fn padding<P: Into<Padding>>(mut self, padding: P) -> Self {
        self.padding = padding.into();
        self
    }

    /// Sets the text size of the [`TextInput`].
    pub fn size(mut self, size: u16) -> Self {
        self.size = Some(size);
        self
    }

    /// Sets the message that should be produced when the [`TextInput`] is
    /// focused and the enter key is pressed.
    pub fn on_submit(mut self, message: Message) -> Self {
        self.on_submit = Some(message);
        self
    }

    /// Sets the style of the [`TextInput`].
    pub fn style(mut self, style: impl Into<Renderer::Style>) -> Self {
        self.style = style.into();
        self
    }

    /// Returns the current [`State`] of the [`TextInput`].
    pub fn state(&self) -> &State {
        self.state
    }
}

impl<'a, Message, Renderer> TextInput<'a, Message, Renderer>
where
    Renderer: self::Renderer,
{
    /// Draws the [`TextInput`] with the given [`Renderer`], overriding its
    /// [`Value`] if provided.
    pub fn draw(
        &self,
        renderer: &mut Renderer,
        layout: Layout<'_>,
        cursor_position: Point,
        value: Option<&Value>,
    ) -> Renderer::Output {
        let value = value.unwrap_or(&self.value);
        let bounds = layout.bounds();
        let text_bounds = layout.children().next().unwrap().bounds();

        if self.is_secure {
            self::Renderer::draw(
                renderer,
                bounds,
                text_bounds,
                cursor_position,
                self.font,
                self.size.unwrap_or(renderer.default_size()),
                &self.placeholder,
                &value.secure(),
                &self.state,
                &self.style,
            )
        } else {
            self::Renderer::draw(
                renderer,
                bounds,
                text_bounds,
                cursor_position,
                self.font,
                self.size.unwrap_or(renderer.default_size()),
                &self.placeholder,
                value,
                &self.state,
                &self.style,
            )
        }
    }
}

impl<'a, Message, Renderer> Widget<Message, Renderer>
    for TextInput<'a, Message, Renderer>
where
    Message: Clone,
    Renderer: self::Renderer,
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
        let text_size = self.size.unwrap_or(renderer.default_size());

        let limits = limits
            .pad(self.padding)
            .width(self.width)
            .max_width(self.max_width)
            .height(Length::Units(text_size));

        let mut text = layout::Node::new(limits.resolve(Size::ZERO));
        text.move_to(Point::new(
            self.padding.left.into(),
            self.padding.top.into(),
        ));

        layout::Node::with_children(text.size().pad(self.padding), vec![text])
    }

    fn on_event(
        &mut self,
        event: Event,
        layout: Layout<'_>,
        cursor_position: Point,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        messages: &mut Vec<Message>,
    ) -> event::Status {
        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
            | Event::Touch(touch::Event::FingerPressed { .. }) => {
                let is_clicked = layout.bounds().contains(cursor_position);

                self.state.is_focused = is_clicked;

                if is_clicked {
                    let text_layout = layout.children().next().unwrap();
                    let target = cursor_position.x - text_layout.bounds().x;

                    let click = mouse::Click::new(
                        cursor_position,
                        self.state.last_click,
                    );

                    match click.kind() {
                        click::Kind::Single => {
                            if target > 0.0 {
                                let value = if self.is_secure {
                                    self.value.secure()
                                } else {
                                    self.value.clone()
                                };

                                let position = renderer.find_cursor_position(
                                    text_layout.bounds(),
                                    self.font,
                                    self.size,
                                    &value,
                                    &self.state,
                                    target,
                                );

                                self.state.cursor.move_to(position);
                            } else {
                                self.state.cursor.move_to(0);
                            }

                            self.state.is_dragging = true;
                        }
                        click::Kind::Double => {
                            if self.is_secure {
                                self.state.cursor.select_all(&self.value);
                            } else {
                                let position = renderer.find_cursor_position(
                                    text_layout.bounds(),
                                    self.font,
                                    self.size,
                                    &self.value,
                                    &self.state,
                                    target,
                                );

                                self.state.cursor.select_range(
                                    self.value.previous_start_of_word(position),
                                    self.value.next_end_of_word(position),
                                );
                            }

                            self.state.is_dragging = false;
                        }
                        click::Kind::Triple => {
                            self.state.cursor.select_all(&self.value);
                            self.state.is_dragging = false;
                        }
                    }

                    self.state.last_click = Some(click);

                    return event::Status::Captured;
                }
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left))
            | Event::Touch(touch::Event::FingerLifted { .. })
            | Event::Touch(touch::Event::FingerLost { .. }) => {
                self.state.is_dragging = false;
            }
            Event::Mouse(mouse::Event::CursorMoved { position })
            | Event::Touch(touch::Event::FingerMoved { position, .. }) => {
                if self.state.is_dragging {
                    let text_layout = layout.children().next().unwrap();
                    let target = position.x - text_layout.bounds().x;

                    if target > 0.0 {
                        let value = if self.is_secure {
                            self.value.secure()
                        } else {
                            self.value.clone()
                        };

                        let position = renderer.find_cursor_position(
                            text_layout.bounds(),
                            self.font,
                            self.size,
                            &value,
                            &self.state,
                            target,
                        );

                        self.state.cursor.select_range(
                            self.state.cursor.start(&value),
                            position,
                        );
                    }

                    return event::Status::Captured;
                }
            }
            Event::Keyboard(keyboard::Event::CharacterReceived(c))
                if self.state.is_focused
                    && self.state.is_pasting.is_none()
                    && !self.state.keyboard_modifiers.command()
                    && !c.is_control() =>
            {
                let mut editor =
                    Editor::new(&mut self.value, &mut self.state.cursor);

                editor.insert(c);

                let message = (self.on_change)(editor.contents());
                messages.push(message);

                return event::Status::Captured;
            }
            Event::Keyboard(keyboard::Event::KeyPressed {
                key_code, ..
            }) if self.state.is_focused => {
                let modifiers = self.state.keyboard_modifiers;

                match key_code {
                    keyboard::KeyCode::Enter => {
                        if let Some(on_submit) = self.on_submit.clone() {
                            messages.push(on_submit);
                        }
                    }
                    keyboard::KeyCode::Backspace => {
                        if platform::is_jump_modifier_pressed(modifiers)
                            && self
                                .state
                                .cursor
                                .selection(&self.value)
                                .is_none()
                        {
                            if self.is_secure {
                                let cursor_pos =
                                    self.state.cursor.end(&self.value);
                                self.state.cursor.select_range(0, cursor_pos);
                            } else {
                                self.state
                                    .cursor
                                    .select_left_by_words(&self.value);
                            }
                        }

                        let mut editor = Editor::new(
                            &mut self.value,
                            &mut self.state.cursor,
                        );

                        editor.backspace();

                        let message = (self.on_change)(editor.contents());
                        messages.push(message);
                    }
                    keyboard::KeyCode::Delete => {
                        if platform::is_jump_modifier_pressed(modifiers)
                            && self
                                .state
                                .cursor
                                .selection(&self.value)
                                .is_none()
                        {
                            if self.is_secure {
                                let cursor_pos =
                                    self.state.cursor.end(&self.value);
                                self.state
                                    .cursor
                                    .select_range(cursor_pos, self.value.len());
                            } else {
                                self.state
                                    .cursor
                                    .select_right_by_words(&self.value);
                            }
                        }

                        let mut editor = Editor::new(
                            &mut self.value,
                            &mut self.state.cursor,
                        );

                        editor.delete();

                        let message = (self.on_change)(editor.contents());
                        messages.push(message);
                    }
                    keyboard::KeyCode::Left => {
                        if platform::is_jump_modifier_pressed(modifiers)
                            && !self.is_secure
                        {
                            if modifiers.shift() {
                                self.state
                                    .cursor
                                    .select_left_by_words(&self.value);
                            } else {
                                self.state
                                    .cursor
                                    .move_left_by_words(&self.value);
                            }
                        } else if modifiers.shift() {
                            self.state.cursor.select_left(&self.value)
                        } else {
                            self.state.cursor.move_left(&self.value);
                        }
                    }
                    keyboard::KeyCode::Right => {
                        if platform::is_jump_modifier_pressed(modifiers)
                            && !self.is_secure
                        {
                            if modifiers.shift() {
                                self.state
                                    .cursor
                                    .select_right_by_words(&self.value);
                            } else {
                                self.state
                                    .cursor
                                    .move_right_by_words(&self.value);
                            }
                        } else if modifiers.shift() {
                            self.state.cursor.select_right(&self.value)
                        } else {
                            self.state.cursor.move_right(&self.value);
                        }
                    }
                    keyboard::KeyCode::Home => {
                        if modifiers.shift() {
                            self.state.cursor.select_range(
                                self.state.cursor.start(&self.value),
                                0,
                            );
                        } else {
                            self.state.cursor.move_to(0);
                        }
                    }
                    keyboard::KeyCode::End => {
                        if modifiers.shift() {
                            self.state.cursor.select_range(
                                self.state.cursor.start(&self.value),
                                self.value.len(),
                            );
                        } else {
                            self.state.cursor.move_to(self.value.len());
                        }
                    }
                    keyboard::KeyCode::C
                        if self.state.keyboard_modifiers.command() =>
                    {
                        match self.state.cursor.selection(&self.value) {
                            Some((start, end)) => {
                                clipboard.write(
                                    self.value.select(start, end).to_string(),
                                );
                            }
                            None => {}
                        }
                    }
                    keyboard::KeyCode::X
                        if self.state.keyboard_modifiers.command() =>
                    {
                        match self.state.cursor.selection(&self.value) {
                            Some((start, end)) => {
                                clipboard.write(
                                    self.value.select(start, end).to_string(),
                                );
                            }
                            None => {}
                        }

                        let mut editor = Editor::new(
                            &mut self.value,
                            &mut self.state.cursor,
                        );

                        editor.delete();

                        let message = (self.on_change)(editor.contents());
                        messages.push(message);
                    }
                    keyboard::KeyCode::V => {
                        if self.state.keyboard_modifiers.command() {
                            let content = match self.state.is_pasting.take() {
                                Some(content) => content,
                                None => {
                                    let content: String = clipboard
                                        .read()
                                        .unwrap_or(String::new())
                                        .chars()
                                        .filter(|c| !c.is_control())
                                        .collect();

                                    Value::new(&content)
                                }
                            };

                            let mut editor = Editor::new(
                                &mut self.value,
                                &mut self.state.cursor,
                            );

                            editor.paste(content.clone());

                            let message = (self.on_change)(editor.contents());
                            messages.push(message);

                            self.state.is_pasting = Some(content);
                        } else {
                            self.state.is_pasting = None;
                        }
                    }
                    keyboard::KeyCode::A
                        if self.state.keyboard_modifiers.command() =>
                    {
                        self.state.cursor.select_all(&self.value);
                    }
                    keyboard::KeyCode::Escape => {
                        self.state.is_focused = false;
                        self.state.is_dragging = false;
                        self.state.is_pasting = None;

                        self.state.keyboard_modifiers =
                            keyboard::Modifiers::default();
                    }
                    _ => {}
                }

                return event::Status::Captured;
            }
            Event::Keyboard(keyboard::Event::KeyReleased {
                key_code, ..
            }) if self.state.is_focused => {
                match key_code {
                    keyboard::KeyCode::V => {
                        self.state.is_pasting = None;
                    }
                    _ => {}
                }

                return event::Status::Captured;
            }
            Event::Keyboard(keyboard::Event::ModifiersChanged(modifiers))
                if self.state.is_focused =>
            {
                self.state.keyboard_modifiers = modifiers;
            }
            _ => {}
        }

        event::Status::Ignored
    }

    fn draw(
        &self,
        renderer: &mut Renderer,
        _defaults: &Renderer::Defaults,
        layout: Layout<'_>,
        cursor_position: Point,
        _viewport: &Rectangle,
    ) -> Renderer::Output {
        self.draw(renderer, layout, cursor_position, None)
    }

    fn hash_layout(&self, state: &mut Hasher) {
        use std::{any::TypeId, hash::Hash};
        struct Marker;
        TypeId::of::<Marker>().hash(state);

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
/// [renderer]: crate::renderer
pub trait Renderer: text::Renderer + Sized {
    /// The style supported by this renderer.
    type Style: Default;

    /// Returns the width of the value of the [`TextInput`].
    fn measure_value(&self, value: &str, size: u16, font: Self::Font) -> f32;

    /// Returns the current horizontal offset of the value of the
    /// [`TextInput`].
    ///
    /// This is the amount of horizontal scrolling applied when the [`Value`]
    /// does not fit the [`TextInput`].
    fn offset(
        &self,
        text_bounds: Rectangle,
        font: Self::Font,
        size: u16,
        value: &Value,
        state: &State,
    ) -> f32;

    /// Draws a [`TextInput`].
    ///
    /// It receives:
    /// - the bounds of the [`TextInput`]
    /// - the bounds of the text (i.e. the current value)
    /// - the cursor position
    /// - the placeholder to show when the value is empty
    /// - the current [`Value`]
    /// - the current [`State`]
    fn draw(
        &mut self,
        bounds: Rectangle,
        text_bounds: Rectangle,
        cursor_position: Point,
        font: Self::Font,
        size: u16,
        placeholder: &str,
        value: &Value,
        state: &State,
        style: &Self::Style,
    ) -> Self::Output;

    /// Computes the position of the text cursor at the given X coordinate of
    /// a [`TextInput`].
    fn find_cursor_position(
        &self,
        text_bounds: Rectangle,
        font: Self::Font,
        size: Option<u16>,
        value: &Value,
        state: &State,
        x: f32,
    ) -> usize {
        let size = size.unwrap_or(self.default_size());

        let offset = self.offset(text_bounds, font, size, &value, &state);

        find_cursor_position(
            self,
            &value,
            font,
            size,
            x + offset,
            0,
            value.len(),
        )
    }
}

impl<'a, Message, Renderer> From<TextInput<'a, Message, Renderer>>
    for Element<'a, Message, Renderer>
where
    Message: 'a + Clone,
    Renderer: 'a + self::Renderer,
{
    fn from(
        text_input: TextInput<'a, Message, Renderer>,
    ) -> Element<'a, Message, Renderer> {
        Element::new(text_input)
    }
}

/// The state of a [`TextInput`].
#[derive(Debug, Default, Clone)]
pub struct State {
    is_focused: bool,
    is_dragging: bool,
    is_pasting: Option<Value>,
    last_click: Option<mouse::Click>,
    cursor: Cursor,
    keyboard_modifiers: keyboard::Modifiers,
    // TODO: Add stateful horizontal scrolling offset
}

impl State {
    /// Creates a new [`State`], representing an unfocused [`TextInput`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a new [`State`], representing a focused [`TextInput`].
    pub fn focused() -> Self {
        Self {
            is_focused: true,
            is_dragging: false,
            is_pasting: None,
            last_click: None,
            cursor: Cursor::default(),
            keyboard_modifiers: keyboard::Modifiers::default(),
        }
    }

    /// Returns whether the [`TextInput`] is currently focused or not.
    pub fn is_focused(&self) -> bool {
        self.is_focused
    }

    /// Returns the [`Cursor`] of the [`TextInput`].
    pub fn cursor(&self) -> Cursor {
        self.cursor
    }

    /// Focuses the [`TextInput`].
    pub fn focus(&mut self) {
        self.is_focused = true;
    }

    /// Unfocuses the [`TextInput`].
    pub fn unfocus(&mut self) {
        self.is_focused = false;
    }

    /// Moves the [`Cursor`] of the [`TextInput`] to the front of the input text.
    pub fn move_cursor_to_front(&mut self) {
        self.cursor.move_to(0);
    }

    /// Moves the [`Cursor`] of the [`TextInput`] to the end of the input text.
    pub fn move_cursor_to_end(&mut self) {
        self.cursor.move_to(usize::MAX);
    }

    /// Moves the [`Cursor`] of the [`TextInput`] to an arbitrary location.
    pub fn move_cursor_to(&mut self, position: usize) {
        self.cursor.move_to(position);
    }

    /// Selects all the content of the [`TextInput`].
    pub fn select_all(&mut self) {
        self.cursor.select_range(0, usize::MAX);
    }
}

// TODO: Reduce allocations
fn find_cursor_position<Renderer: self::Renderer>(
    renderer: &Renderer,
    value: &Value,
    font: Renderer::Font,
    size: u16,
    target: f32,
    start: usize,
    end: usize,
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
            value,
            font,
            size,
            target,
            start,
            start + index,
        )
    } else {
        find_cursor_position(
            renderer,
            value,
            font,
            size,
            target,
            start + index + 1,
            end,
        )
    }
}

mod platform {
    use crate::keyboard;

    pub fn is_jump_modifier_pressed(modifiers: keyboard::Modifiers) -> bool {
        if cfg!(target_os = "macos") {
            modifiers.alt()
        } else {
            modifiers.control()
        }
    }
}
