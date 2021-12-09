//! Display fields that can be filled with text.
//!
//! A [`TextInput`] has some local [`State`].
mod editor;
mod value;

pub mod cursor;

pub use cursor::Cursor;
pub use value::Value;

use editor::Editor;

use crate::alignment;
use crate::event::{self, Event};
use crate::keyboard;
use crate::layout;
use crate::mouse::{self, click};
use crate::renderer;
use crate::text::{self, Text};
use crate::touch;
use crate::{
    Clipboard, Color, Element, Hasher, Layout, Length, Padding, Point,
    Rectangle, Shell, Size, Vector, Widget,
};

use std::u32;

pub use iced_style::text_input::{Style, StyleSheet};

/// A field that can be filled with text.
///
/// # Example
/// ```
/// # use iced_native::renderer::Null;
/// # use iced_native::widget::text_input;
/// #
/// # pub type TextInput<'a, Message> = iced_native::widget::TextInput<'a, Message, Null>;
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
pub struct TextInput<'a, Message, Renderer: text::Renderer> {
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
    style_sheet: Box<dyn StyleSheet + 'a>,
}

impl<'a, Message, Renderer> TextInput<'a, Message, Renderer>
where
    Message: Clone,
    Renderer: text::Renderer,
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
            style_sheet: Default::default(),
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
    pub fn style(
        mut self,
        style_sheet: impl Into<Box<dyn StyleSheet + 'a>>,
    ) -> Self {
        self.style_sheet = style_sheet.into();
        self
    }

    /// Returns the current [`State`] of the [`TextInput`].
    pub fn state(&self) -> &State {
        self.state
    }
}

impl<'a, Message, Renderer> TextInput<'a, Message, Renderer>
where
    Renderer: text::Renderer,
{
    /// Draws the [`TextInput`] with the given [`Renderer`], overriding its
    /// [`Value`] if provided.
    pub fn draw(
        &self,
        renderer: &mut Renderer,
        layout: Layout<'_>,
        cursor_position: Point,
        value: Option<&Value>,
    ) {
        let value = value.unwrap_or(&self.value);
        let secure_value = self.is_secure.then(|| value.secure());
        let value = secure_value.as_ref().unwrap_or(&value);

        let bounds = layout.bounds();
        let text_bounds = layout.children().next().unwrap().bounds();

        let is_mouse_over = bounds.contains(cursor_position);

        let style = if self.state.is_focused() {
            self.style_sheet.focused()
        } else if is_mouse_over {
            self.style_sheet.hovered()
        } else {
            self.style_sheet.active()
        };

        renderer.fill_quad(
            renderer::Quad {
                bounds,
                border_radius: style.border_radius,
                border_width: style.border_width,
                border_color: style.border_color,
            },
            style.background,
        );

        let text = value.to_string();
        let size = self.size.unwrap_or(renderer.default_size());

        let (cursor, offset) = if self.state.is_focused() {
            match self.state.cursor.state(&value) {
                cursor::State::Index(position) => {
                    let (text_value_width, offset) =
                        measure_cursor_and_scroll_offset(
                            renderer,
                            text_bounds,
                            &value,
                            size,
                            position,
                            self.font,
                        );

                    (
                        Some((
                            renderer::Quad {
                                bounds: Rectangle {
                                    x: text_bounds.x + text_value_width,
                                    y: text_bounds.y,
                                    width: 1.0,
                                    height: text_bounds.height,
                                },
                                border_radius: 0.0,
                                border_width: 0.0,
                                border_color: Color::TRANSPARENT,
                            },
                            self.style_sheet.value_color(),
                        )),
                        offset,
                    )
                }
                cursor::State::Selection { start, end } => {
                    let left = start.min(end);
                    let right = end.max(start);

                    let (left_position, left_offset) =
                        measure_cursor_and_scroll_offset(
                            renderer,
                            text_bounds,
                            &value,
                            size,
                            left,
                            self.font,
                        );

                    let (right_position, right_offset) =
                        measure_cursor_and_scroll_offset(
                            renderer,
                            text_bounds,
                            &value,
                            size,
                            right,
                            self.font,
                        );

                    let width = right_position - left_position;

                    (
                        Some((
                            renderer::Quad {
                                bounds: Rectangle {
                                    x: text_bounds.x + left_position,
                                    y: text_bounds.y,
                                    width,
                                    height: text_bounds.height,
                                },
                                border_radius: 0.0,
                                border_width: 0.0,
                                border_color: Color::TRANSPARENT,
                            },
                            self.style_sheet.selection_color(),
                        )),
                        if end == right {
                            right_offset
                        } else {
                            left_offset
                        },
                    )
                }
            }
        } else {
            (None, 0.0)
        };

        let text_width = renderer.measure_width(
            if text.is_empty() {
                &self.placeholder
            } else {
                &text
            },
            size,
            self.font,
        );

        let render = |renderer: &mut Renderer| {
            if let Some((cursor, color)) = cursor {
                renderer.fill_quad(cursor, color);
            }

            renderer.fill_text(Text {
                content: if text.is_empty() {
                    &self.placeholder
                } else {
                    &text
                },
                color: if text.is_empty() {
                    self.style_sheet.placeholder_color()
                } else {
                    self.style_sheet.value_color()
                },
                font: self.font,
                bounds: Rectangle {
                    y: text_bounds.center_y(),
                    width: f32::INFINITY,
                    ..text_bounds
                },
                size: f32::from(size),
                horizontal_alignment: alignment::Horizontal::Left,
                vertical_alignment: alignment::Vertical::Center,
            });
        };

        if text_width > text_bounds.width {
            renderer.with_layer(text_bounds, |renderer| {
                renderer.with_translation(Vector::new(-offset, 0.0), render)
            });
        } else {
            render(renderer);
        }
    }
}

impl<'a, Message, Renderer> Widget<Message, Renderer>
    for TextInput<'a, Message, Renderer>
where
    Message: Clone,
    Renderer: text::Renderer,
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
        shell: &mut Shell<'_, Message>,
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
                            let position = if target > 0.0 {
                                let value = if self.is_secure {
                                    self.value.secure()
                                } else {
                                    self.value.clone()
                                };

                                find_cursor_position(
                                    renderer,
                                    text_layout.bounds(),
                                    self.font,
                                    self.size,
                                    &value,
                                    &self.state,
                                    target,
                                )
                            } else {
                                None
                            };

                            self.state.cursor.move_to(position.unwrap_or(0));
                            self.state.is_dragging = true;
                        }
                        click::Kind::Double => {
                            if self.is_secure {
                                self.state.cursor.select_all(&self.value);
                            } else {
                                let position = find_cursor_position(
                                    renderer,
                                    text_layout.bounds(),
                                    self.font,
                                    self.size,
                                    &self.value,
                                    &self.state,
                                    target,
                                )
                                .unwrap_or(0);

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

                    let value = if self.is_secure {
                        self.value.secure()
                    } else {
                        self.value.clone()
                    };

                    let position = find_cursor_position(
                        renderer,
                        text_layout.bounds(),
                        self.font,
                        self.size,
                        &value,
                        &self.state,
                        target,
                    )
                    .unwrap_or(0);

                    self.state.cursor.select_range(
                        self.state.cursor.start(&value),
                        position,
                    );

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
                shell.publish(message);

                return event::Status::Captured;
            }
            Event::Keyboard(keyboard::Event::KeyPressed {
                key_code, ..
            }) if self.state.is_focused => {
                let modifiers = self.state.keyboard_modifiers;

                match key_code {
                    keyboard::KeyCode::Enter
                    | keyboard::KeyCode::NumpadEnter => {
                        if let Some(on_submit) = self.on_submit.clone() {
                            shell.publish(on_submit);
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
                        shell.publish(message);
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
                        shell.publish(message);
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
                        shell.publish(message);
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
                            shell.publish(message);

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
                    keyboard::KeyCode::Tab
                    | keyboard::KeyCode::Up
                    | keyboard::KeyCode::Down => {
                        return event::Status::Ignored;
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
                    keyboard::KeyCode::Tab
                    | keyboard::KeyCode::Up
                    | keyboard::KeyCode::Down => {
                        return event::Status::Ignored;
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

    fn mouse_interaction(
        &self,
        layout: Layout<'_>,
        cursor_position: Point,
        _viewport: &Rectangle,
    ) -> mouse::Interaction {
        if layout.bounds().contains(cursor_position) {
            mouse::Interaction::Text
        } else {
            mouse::Interaction::default()
        }
    }

    fn draw(
        &self,
        renderer: &mut Renderer,
        _style: &renderer::Style,
        layout: Layout<'_>,
        cursor_position: Point,
        _viewport: &Rectangle,
    ) {
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

impl<'a, Message, Renderer> From<TextInput<'a, Message, Renderer>>
    for Element<'a, Message, Renderer>
where
    Message: 'a + Clone,
    Renderer: 'a + text::Renderer,
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

fn offset<Renderer>(
    renderer: &Renderer,
    text_bounds: Rectangle,
    font: Renderer::Font,
    size: u16,
    value: &Value,
    state: &State,
) -> f32
where
    Renderer: text::Renderer,
{
    if state.is_focused() {
        let cursor = state.cursor();

        let focus_position = match cursor.state(value) {
            cursor::State::Index(i) => i,
            cursor::State::Selection { end, .. } => end,
        };

        let (_, offset) = measure_cursor_and_scroll_offset(
            renderer,
            text_bounds,
            value,
            size,
            focus_position,
            font,
        );

        offset
    } else {
        0.0
    }
}

fn measure_cursor_and_scroll_offset<Renderer>(
    renderer: &Renderer,
    text_bounds: Rectangle,
    value: &Value,
    size: u16,
    cursor_index: usize,
    font: Renderer::Font,
) -> (f32, f32)
where
    Renderer: text::Renderer,
{
    let text_before_cursor = value.until(cursor_index).to_string();

    let text_value_width =
        renderer.measure_width(&text_before_cursor, size, font);

    let offset = ((text_value_width + 5.0) - text_bounds.width).max(0.0);

    (text_value_width, offset)
}

/// Computes the position of the text cursor at the given X coordinate of
/// a [`TextInput`].
fn find_cursor_position<Renderer>(
    renderer: &Renderer,
    text_bounds: Rectangle,
    font: Renderer::Font,
    size: Option<u16>,
    value: &Value,
    state: &State,
    x: f32,
) -> Option<usize>
where
    Renderer: text::Renderer,
{
    let size = size.unwrap_or(renderer.default_size());

    let offset = offset(renderer, text_bounds, font, size, &value, &state);

    renderer
        .hit_test(
            &value.to_string(),
            size.into(),
            font,
            Size::INFINITY,
            Point::new(x + offset, text_bounds.height / 2.0),
            true,
        )
        .map(text::Hit::cursor)
}
