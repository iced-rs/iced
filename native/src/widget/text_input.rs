//! Display fields that can be filled with text.
//!
//! A [`TextInput`] has some local [`State`].
pub mod cursor;
mod editor;
mod ime_state;
mod value;

pub use cursor::Cursor;
pub use value::Value;

use editor::Editor;

use crate::event::{self, Event};
use crate::keyboard;
use crate::layout;
use crate::mouse::{self, click};
use crate::renderer;
use crate::text::{self, Text};
use crate::time::{Duration, Instant};
use crate::touch;
use crate::widget;
use crate::widget::operation::{self, Operation};
use crate::widget::tree::{self, Tree};
use crate::window;
use crate::{alignment, IME};
use crate::{
    Clipboard, Color, Command, Element, Layout, Length, Padding, Pixels, Point,
    Rectangle, Shell, Size, Vector, Widget,
};
pub use iced_style::text_input::{Appearance, StyleSheet};

use self::ime_state::IMEState;
/// A field that can be filled with text.
///
/// # Example
/// ```
/// # pub type TextInput<'a, Message> = iced_native::widget::TextInput<'a, Message, iced_native::renderer::Null>;
/// #[derive(Debug, Clone)]
/// enum Message {
///     TextInputChanged(String),
/// }
///
/// let value = "Some text";
///
/// let input = TextInput::new(
///     "This is the placeholder...",
///     value,
/// )
/// .on_input(Message::TextInputChanged)
/// .padding(10);
/// ```
/// ![Text input drawn by `iced_wgpu`](https://github.com/iced-rs/iced/blob/7760618fb112074bc40b148944521f312152012a/docs/images/text_input.png?raw=true)
#[allow(missing_debug_implementations)]
pub struct TextInput<'a, Message, Renderer>
where
    Renderer: text::Renderer,
    Renderer::Theme: StyleSheet,
{
    id: Option<Id>,
    placeholder: String,
    value: Value,
    is_secure: bool,
    font: Renderer::Font,
    width: Length,
    padding: Padding,
    size: Option<f32>,
    on_input: Option<Box<dyn Fn(String) -> Message + 'a>>,
    on_paste: Option<Box<dyn Fn(String) -> Message + 'a>>,
    on_submit: Option<Message>,
    icon: Option<Icon<Renderer::Font>>,
    style: <Renderer::Theme as StyleSheet>::Style,
}

impl<'a, Message, Renderer> TextInput<'a, Message, Renderer>
where
    Message: Clone,
    Renderer: text::Renderer,
    Renderer::Theme: StyleSheet,
{
    /// Creates a new [`TextInput`].
    ///
    /// It expects:
    /// - a placeholder,
    /// - the current value
    pub fn new(placeholder: &str, value: &str) -> Self {
        TextInput {
            id: None,
            placeholder: String::from(placeholder),
            value: Value::new(value),
            is_secure: false,
            font: Default::default(),
            width: Length::Fill,
            padding: Padding::new(5.0),
            size: None,
            on_input: None,
            on_paste: None,
            on_submit: None,
            icon: None,
            style: Default::default(),
        }
    }

    /// Sets the [`Id`] of the [`TextInput`].
    pub fn id(mut self, id: Id) -> Self {
        self.id = Some(id);
        self
    }

    /// Converts the [`TextInput`] into a secure password input.
    pub fn password(mut self) -> Self {
        self.is_secure = true;
        self
    }

    /// Sets the message that should be produced when some text is typed into
    /// the [`TextInput`].
    ///
    /// If this method is not called, the [`TextInput`] will be disabled.
    pub fn on_input<F>(mut self, callback: F) -> Self
    where
        F: 'a + Fn(String) -> Message,
    {
        self.on_input = Some(Box::new(callback));
        self
    }

    /// Sets the message that should be produced when the [`TextInput`] is
    /// focused and the enter key is pressed.
    pub fn on_submit(mut self, message: Message) -> Self {
        self.on_submit = Some(message);
        self
    }

    /// Sets the message that should be produced when some text is pasted into
    /// the [`TextInput`].
    pub fn on_paste(
        mut self,
        on_paste: impl Fn(String) -> Message + 'a,
    ) -> Self {
        self.on_paste = Some(Box::new(on_paste));
        self
    }

    /// Sets the [`Font`] of the [`TextInput`].
    ///
    /// [`Font`]: text::Renderer::Font
    pub fn font(mut self, font: Renderer::Font) -> Self {
        self.font = font;
        self
    }

    /// Sets the [`Icon`] of the [`TextInput`].
    pub fn icon(mut self, icon: Icon<Renderer::Font>) -> Self {
        self.icon = Some(icon);
        self
    }

    /// Sets the width of the [`TextInput`].
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    /// Sets the [`Padding`] of the [`TextInput`].
    pub fn padding<P: Into<Padding>>(mut self, padding: P) -> Self {
        self.padding = padding.into();
        self
    }

    /// Sets the text size of the [`TextInput`].
    pub fn size(mut self, size: impl Into<Pixels>) -> Self {
        self.size = Some(size.into().0);
        self
    }

    /// Sets the style of the [`TextInput`].
    pub fn style(
        mut self,
        style: impl Into<<Renderer::Theme as StyleSheet>::Style>,
    ) -> Self {
        self.style = style.into();
        self
    }

    /// Draws the [`TextInput`] with the given [`Renderer`], overriding its
    /// [`Value`] if provided.
    ///
    /// [`Renderer`]: text::Renderer
    pub fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Renderer::Theme,
        layout: Layout<'_>,
        cursor_position: Point,
        value: Option<&Value>,
    ) {
        draw(
            renderer,
            theme,
            layout,
            cursor_position,
            tree.state.downcast_ref::<State>(),
            value.unwrap_or(&self.value),
            &self.placeholder,
            self.size,
            &self.font,
            self.on_input.is_none(),
            self.is_secure,
            self.icon.as_ref(),
            &self.style,
        )
    }
}

impl<'a, Message, Renderer> Widget<Message, Renderer>
    for TextInput<'a, Message, Renderer>
where
    Message: Clone,
    Renderer: text::Renderer,
    Renderer::Theme: StyleSheet,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State::new())
    }

    fn diff(&self, tree: &mut Tree) {
        let state = tree.state.downcast_mut::<State>();

        // Unfocus text input if it becomes disabled
        if self.on_input.is_none() {
            state.last_click = None;
            state.is_focused = None;
            state.is_pasting = None;
            state.is_dragging = false;
        }
    }

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
        layout(
            renderer,
            limits,
            self.width,
            self.padding,
            self.size,
            self.icon.as_ref(),
        )
    }

    fn operate(
        &self,
        tree: &mut Tree,
        _layout: Layout<'_>,
        _renderer: &Renderer,
        operation: &mut dyn Operation<Message>,
    ) {
        let state = tree.state.downcast_mut::<State>();

        operation.focusable(state, self.id.as_ref().map(|id| &id.0));
        operation.text_input(state, self.id.as_ref().map(|id| &id.0));
    }

    fn on_event(
        &mut self,
        tree: &mut Tree,
        event: Event,
        layout: Layout<'_>,
        cursor_position: Point,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        ime: &dyn IME,
        shell: &mut Shell<'_, Message>,
    ) -> event::Status {
        update(
            event,
            layout,
            cursor_position,
            renderer,
            clipboard,
            ime,
            shell,
            &mut self.value,
            self.size,
            &self.font,
            self.is_secure,
            self.on_input.as_deref(),
            self.on_paste.as_deref(),
            &self.on_submit,
            || tree.state.downcast_mut::<State>(),
        )
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Renderer::Theme,
        _style: &renderer::Style,
        layout: Layout<'_>,
        cursor_position: Point,
        _viewport: &Rectangle,
    ) {
        draw(
            renderer,
            theme,
            layout,
            cursor_position,
            tree.state.downcast_ref::<State>(),
            &self.value,
            &self.placeholder,
            self.size,
            &self.font,
            self.on_input.is_none(),
            self.is_secure,
            self.icon.as_ref(),
            &self.style,
        )
    }

    fn mouse_interaction(
        &self,
        _state: &Tree,
        layout: Layout<'_>,
        cursor_position: Point,
        _viewport: &Rectangle,
        _renderer: &Renderer,
    ) -> mouse::Interaction {
        mouse_interaction(layout, cursor_position, self.on_input.is_none())
    }
}

impl<'a, Message, Renderer> From<TextInput<'a, Message, Renderer>>
    for Element<'a, Message, Renderer>
where
    Message: 'a + Clone,
    Renderer: 'a + text::Renderer,
    Renderer::Theme: StyleSheet,
{
    fn from(
        text_input: TextInput<'a, Message, Renderer>,
    ) -> Element<'a, Message, Renderer> {
        Element::new(text_input)
    }
}

/// The content of the [`Icon`].
#[derive(Debug, Clone)]
pub struct Icon<Font> {
    /// The font that will be used to display the `code_point`.
    pub font: Font,
    /// The unicode code point that will be used as the icon.
    pub code_point: char,
    /// The font size of the content.
    pub size: Option<f32>,
    /// The spacing between the [`Icon`] and the text in a [`TextInput`].
    pub spacing: f32,
    /// The side of a [`TextInput`] where to display the [`Icon`].
    pub side: Side,
}

/// The side of a [`TextInput`].
#[derive(Debug, Clone)]
pub enum Side {
    /// The left side of a [`TextInput`].
    Left,
    /// The right side of a [`TextInput`].
    Right,
}

/// The identifier of a [`TextInput`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Id(widget::Id);

impl Id {
    /// Creates a custom [`Id`].
    pub fn new(id: impl Into<std::borrow::Cow<'static, str>>) -> Self {
        Self(widget::Id::new(id))
    }

    /// Creates a unique [`Id`].
    ///
    /// This function produces a different [`Id`] every time it is called.
    pub fn unique() -> Self {
        Self(widget::Id::unique())
    }
}

impl From<Id> for widget::Id {
    fn from(id: Id) -> Self {
        id.0
    }
}

/// Produces a [`Command`] that focuses the [`TextInput`] with the given [`Id`].
pub fn focus<Message: 'static>(id: Id) -> Command<Message> {
    Command::widget(operation::focusable::focus(id.0))
}

/// Produces a [`Command`] that moves the cursor of the [`TextInput`] with the given [`Id`] to the
/// end.
pub fn move_cursor_to_end<Message: 'static>(id: Id) -> Command<Message> {
    Command::widget(operation::text_input::move_cursor_to_end(id.0))
}

/// Produces a [`Command`] that moves the cursor of the [`TextInput`] with the given [`Id`] to the
/// front.
pub fn move_cursor_to_front<Message: 'static>(id: Id) -> Command<Message> {
    Command::widget(operation::text_input::move_cursor_to_front(id.0))
}

/// Produces a [`Command`] that moves the cursor of the [`TextInput`] with the given [`Id`] to the
/// provided position.
pub fn move_cursor_to<Message: 'static>(
    id: Id,
    position: usize,
) -> Command<Message> {
    Command::widget(operation::text_input::move_cursor_to(id.0, position))
}

/// Produces a [`Command`] that selects all the content of the [`TextInput`] with the given [`Id`].
pub fn select_all<Message: 'static>(id: Id) -> Command<Message> {
    Command::widget(operation::text_input::select_all(id.0))
}

/// Computes the layout of a [`TextInput`].
pub fn layout<Renderer>(
    renderer: &Renderer,
    limits: &layout::Limits,
    width: Length,
    padding: Padding,
    size: Option<f32>,
    icon: Option<&Icon<Renderer::Font>>,
) -> layout::Node
where
    Renderer: text::Renderer,
{
    let text_size = size.unwrap_or_else(|| renderer.default_size());

    let padding = padding.fit(Size::ZERO, limits.max());
    let limits = limits.width(width).pad(padding).height(text_size);

    let text_bounds = limits.resolve(Size::ZERO);

    if let Some(icon) = icon {
        let icon_width = renderer.measure_width(
            &icon.code_point.to_string(),
            icon.size.unwrap_or_else(|| renderer.default_size()),
            icon.font.clone(),
        );

        let mut text_node = layout::Node::new(
            text_bounds - Size::new(icon_width + icon.spacing, 0.0),
        );

        let mut icon_node =
            layout::Node::new(Size::new(icon_width, text_bounds.height));

        match icon.side {
            Side::Left => {
                text_node.move_to(Point::new(
                    padding.left + icon_width + icon.spacing,
                    padding.top,
                ));

                icon_node.move_to(Point::new(padding.left, padding.top));
            }
            Side::Right => {
                text_node.move_to(Point::new(padding.left, padding.top));

                icon_node.move_to(Point::new(
                    padding.left + text_bounds.width - icon_width,
                    padding.top,
                ));
            }
        };

        layout::Node::with_children(
            text_bounds.pad(padding),
            vec![text_node, icon_node],
        )
    } else {
        let mut text = layout::Node::new(text_bounds);
        text.move_to(Point::new(padding.left, padding.top));

        layout::Node::with_children(text_bounds.pad(padding), vec![text])
    }
}

/// Processes an [`Event`] and updates the [`State`] of a [`TextInput`]
/// accordingly.
pub fn update<'a, Message, Renderer>(
    event: Event,
    layout: Layout<'_>,
    cursor_position: Point,
    renderer: &Renderer,
    clipboard: &mut dyn Clipboard,
    ime: &dyn IME,
    shell: &mut Shell<'_, Message>,
    value: &mut Value,
    size: Option<f32>,
    font: &Renderer::Font,
    is_secure: bool,
    on_input: Option<&dyn Fn(String) -> Message>,
    on_paste: Option<&dyn Fn(String) -> Message>,
    on_submit: &Option<Message>,
    state: impl FnOnce() -> &'a mut State,
) -> event::Status
where
    Message: Clone,
    Renderer: text::Renderer,
{
    let state = state();
    if state.is_focused.is_some() {
        if is_secure {
            ime.password_mode();
        } else {
            ime.inside();
        }
    }
    match event {
        Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
        | Event::Touch(touch::Event::FingerPressed { .. }) => {
            let is_clicked =
                layout.bounds().contains(cursor_position) && on_input.is_some();
            // if gain focus enable ime
            let focus_gained = state.is_focused.is_none() && is_clicked;
            let focus_lost = state.is_focused.is_some() && !is_clicked;
            state.is_focused = if is_clicked {
                state.is_focused.or_else(|| {
                    let now = Instant::now();

                    Some(Focus {
                        updated_at: now,
                        now,
                    })
                })
            } else {
                None
            };
            if is_clicked {
                let text_layout = layout.children().next().unwrap();
                let text_bounds = text_layout.bounds();
                let target = cursor_position.x - text_layout.bounds().x;

                let click =
                    mouse::Click::new(cursor_position, state.last_click);

                match click.kind() {
                    click::Kind::Single => {
                        let position = if target > 0.0 {
                            let value = if is_secure {
                                value.secure()
                            } else {
                                value.clone()
                            };

                            find_cursor_position(
                                renderer,
                                text_layout.bounds(),
                                font.clone(),
                                size,
                                &value,
                                state,
                                target,
                            )
                        } else {
                            None
                        }
                        .unwrap_or(0);

                        if state.keyboard_modifiers.shift() {
                            state.cursor.select_range(
                                state.cursor.start(value),
                                position,
                            );
                        } else {
                            state.cursor.move_to(position);
                        }
                        state.is_dragging = true;
                    }
                    click::Kind::Double => {
                        if is_secure {
                            state.cursor.select_all(value);
                        } else {
                            let position = find_cursor_position(
                                renderer,
                                text_layout.bounds(),
                                font.clone(),
                                size,
                                value,
                                state,
                                target,
                            )
                            .unwrap_or(0);
                            state.cursor.select_range(
                                value.previous_start_of_word(position),
                                value.next_end_of_word(position),
                            );
                        }

                        state.is_dragging = false;
                    }
                    click::Kind::Triple => {
                        state.cursor.select_all(value);
                        state.is_dragging = false;
                    }
                }

                state.last_click = Some(click);
                if !is_secure && focus_gained {
                    let position = state.cursor.start(value);

                    // calcurate where we need to place candidate window.
                    let size = size.unwrap_or_else(|| renderer.default_size());

                    let position = measure_cursor_and_scroll_offset(
                        renderer,
                        text_bounds,
                        value,
                        size,
                        position,
                        font.clone(),
                    );
                    let position = (
                        (text_bounds.x + position.0) as i32,
                        (text_bounds.y) as i32 + size as i32,
                    );

                    ime.set_ime_position(position.0, position.1);
                }
            } else if focus_lost {
                let mut editor = Editor::new(value, &mut state.cursor);
                ime.outside();
                if let Some((old_ime_state, on_input)) =
                    state.ime_state.take().zip(on_input)
                {
                    old_ime_state
                        .preedit_text()
                        .chars()
                        .for_each(|ch| editor.insert(ch));
                    let message = (on_input)(editor.contents());
                    shell.publish(message);
                }
            }

            return event::Status::Captured;
        }
        Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left))
        | Event::Touch(touch::Event::FingerLifted { .. })
        | Event::Touch(touch::Event::FingerLost { .. }) => {
            state.is_dragging = false;
        }
        Event::Mouse(mouse::Event::CursorMoved { position })
        | Event::Touch(touch::Event::FingerMoved { position, .. }) => {
            if state.is_dragging {
                let text_layout = layout.children().next().unwrap();
                let target = position.x - text_layout.bounds().x;

                let value = if is_secure {
                    value.secure()
                } else {
                    value.clone()
                };

                let position = find_cursor_position(
                    renderer,
                    text_layout.bounds(),
                    font.clone(),
                    size,
                    &value,
                    state,
                    target,
                )
                .unwrap_or(0);

                state
                    .cursor
                    .select_range(state.cursor.start(&value), position);

                return event::Status::Captured;
            }
        }
        Event::Keyboard(keyboard::Event::CharacterReceived(c)) => {
            if let Some(focus) = &mut state.is_focused {
                let Some(on_input) = on_input else { return event::Status::Ignored };

                if state.is_pasting.is_none()
                    && !state.keyboard_modifiers.command()
                    && !c.is_control()
                {
                    let mut editor = Editor::new(value, &mut state.cursor);

                    editor.insert(c);

                    let message = (on_input)(editor.contents());
                    shell.publish(message);

                    focus.updated_at = Instant::now();

                    return event::Status::Captured;
                }
            }
        }
        Event::Keyboard(keyboard::Event::KeyPressed { key_code, .. }) => {
            if let Some(focus) = &mut state.is_focused {
                let Some(on_input) = on_input else { return event::Status::Ignored };

                let modifiers = state.keyboard_modifiers;
                focus.updated_at = Instant::now();

                match key_code {
                    keyboard::KeyCode::Enter
                    | keyboard::KeyCode::NumpadEnter => {
                        if let Some(on_submit) = on_submit.clone() {
                            shell.publish(on_submit);
                        }
                    }
                    keyboard::KeyCode::Backspace => {
                        if platform::is_jump_modifier_pressed(modifiers)
                            && state.cursor.selection(value).is_none()
                        {
                            if is_secure {
                                let cursor_pos = state.cursor.end(value);
                                state.cursor.select_range(0, cursor_pos);
                            } else {
                                state.cursor.select_left_by_words(value);
                            }
                        }

                        let mut editor = Editor::new(value, &mut state.cursor);
                        editor.backspace();

                        let message = (on_input)(editor.contents());
                        shell.publish(message);
                    }
                    keyboard::KeyCode::Delete => {
                        if platform::is_jump_modifier_pressed(modifiers)
                            && state.cursor.selection(value).is_none()
                        {
                            if is_secure {
                                let cursor_pos = state.cursor.end(value);
                                state
                                    .cursor
                                    .select_range(cursor_pos, value.len());
                            } else {
                                state.cursor.select_right_by_words(value);
                            }
                        }

                        let mut editor = Editor::new(value, &mut state.cursor);
                        editor.delete();

                        let message = (on_input)(editor.contents());
                        shell.publish(message);
                    }
                    keyboard::KeyCode::Left => {
                        if platform::is_jump_modifier_pressed(modifiers)
                            && !is_secure
                        {
                            if modifiers.shift() {
                                state.cursor.select_left_by_words(value);
                            } else {
                                state.cursor.move_left_by_words(value);
                            }
                        } else if modifiers.shift() {
                            state.cursor.select_left(value)
                        } else {
                            state.cursor.move_left(value);
                        }
                    }
                    keyboard::KeyCode::Right => {
                        if platform::is_jump_modifier_pressed(modifiers)
                            && !is_secure
                        {
                            if modifiers.shift() {
                                state.cursor.select_right_by_words(value);
                            } else {
                                state.cursor.move_right_by_words(value);
                            }
                        } else if modifiers.shift() {
                            state.cursor.select_right(value)
                        } else {
                            state.cursor.move_right(value);
                        }
                    }
                    keyboard::KeyCode::Home => {
                        if modifiers.shift() {
                            state
                                .cursor
                                .select_range(state.cursor.start(value), 0);
                        } else {
                            state.cursor.move_to(0);
                        }
                    }
                    keyboard::KeyCode::End => {
                        if modifiers.shift() {
                            state.cursor.select_range(
                                state.cursor.start(value),
                                value.len(),
                            );
                        } else {
                            state.cursor.move_to(value.len());
                        }
                    }
                    keyboard::KeyCode::C
                        if state.keyboard_modifiers.command() =>
                    {
                        if let Some((start, end)) =
                            state.cursor.selection(value)
                        {
                            clipboard
                                .write(value.select(start, end).to_string());
                        }
                    }
                    keyboard::KeyCode::X
                        if state.keyboard_modifiers.command() =>
                    {
                        if let Some((start, end)) =
                            state.cursor.selection(value)
                        {
                            clipboard
                                .write(value.select(start, end).to_string());
                        }

                        let mut editor = Editor::new(value, &mut state.cursor);
                        editor.delete();

                        let message = (on_input)(editor.contents());
                        shell.publish(message);
                    }
                    keyboard::KeyCode::V => {
                        if state.keyboard_modifiers.command() {
                            let content = match state.is_pasting.take() {
                                Some(content) => content,
                                None => {
                                    let content: String = clipboard
                                        .read()
                                        .unwrap_or_default()
                                        .chars()
                                        .filter(|c| !c.is_control())
                                        .collect();

                                    Value::new(&content)
                                }
                            };

                            let mut editor =
                                Editor::new(value, &mut state.cursor);

                            editor.paste(content.clone());

                            let message = if let Some(paste) = &on_paste {
                                (paste)(editor.contents())
                            } else {
                                (on_input)(editor.contents())
                            };
                            shell.publish(message);

                            state.is_pasting = Some(content);
                        } else {
                            state.is_pasting = None;
                        }
                    }
                    keyboard::KeyCode::A
                        if state.keyboard_modifiers.command() =>
                    {
                        state.cursor.select_all(value);
                    }
                    keyboard::KeyCode::Escape => {
                        state.is_focused = None;
                        state.is_dragging = false;
                        state.is_pasting = None;

                        state.keyboard_modifiers =
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
        }
        Event::Keyboard(keyboard::Event::KeyReleased { key_code, .. }) => {
            if state.is_focused.is_some() {
                match key_code {
                    keyboard::KeyCode::V => {
                        state.is_pasting = None;
                    }
                    keyboard::KeyCode::Tab
                    | keyboard::KeyCode::Up
                    | keyboard::KeyCode::Down => {
                        return event::Status::Ignored;
                    }
                    _ => {}
                }

                return event::Status::Captured;
            } else {
                state.is_pasting = None;
            }
        }
        Event::Keyboard(keyboard::Event::ModifiersChanged(modifiers)) => {
            state.keyboard_modifiers = modifiers;
        }
        Event::Window(window::Event::RedrawRequested(now)) => {
            if let Some(focus) = &mut state.is_focused {
                focus.now = now;

                let millis_until_redraw = CURSOR_BLINK_INTERVAL_MILLIS
                    - (now - focus.updated_at).as_millis()
                        % CURSOR_BLINK_INTERVAL_MILLIS;

                shell.request_redraw(window::RedrawRequest::At(
                    now + Duration::from_millis(millis_until_redraw as u64),
                ));
            }
        }
        Event::Keyboard(keyboard::Event::IMECommit(text)) => {
            if let Some(on_input) = (state.is_pasting.is_none()
                && !state.keyboard_modifiers.command()
                && !is_secure
                && state.is_focused.is_some()
                && state.ime_state.is_some())
            .then(|| on_input)
            .flatten()
            {
                let mut editor = Editor::new(value, &mut state.cursor);

                for ch in text.chars() {
                    editor.insert(ch);
                }

                let message = (on_input)(editor.contents());
                shell.publish(message);
                #[cfg(target_os = "macos")]
                {
                    let text_bounds =
                        layout.children().next().unwrap().bounds();
                    let size = size.unwrap_or_else(|| renderer.default_size());

                    let width = renderer.measure_width(
                        &editor.contents(),
                        size,
                        font.clone(),
                    );
                    let (x, y) = (
                        (text_bounds.x + width) as i32,
                        (text_bounds.y) as i32 + size as i32,
                    );
                    ime.set_ime_position_with_reenable(x, y);
                }
                state.ime_state = None;
                return event::Status::Captured;
            }
        }

        Event::Keyboard(keyboard::Event::IMEEnabled) => {
            if state.is_focused.is_some() {
                let _ = state.ime_state.replace(IMEState::default());
                return event::Status::Captured;
            }
        }
        Event::Keyboard(keyboard::Event::IMEPreedit(text, range)) => {
            if state.is_focused.is_none() || is_secure {
                return event::Status::Ignored;
            }
            // calcurate where we need to place candidate window.
            let text_bounds = layout.children().next().unwrap().bounds();
            let size = size.unwrap_or_else(|| renderer.default_size());

            let chars_from_left = state.cursor.start(value);

            let before_preedit_text = value
                .to_string()
                .chars()
                .take(chars_from_left)
                .fold(String::new(), |mut buffer, ch| {
                    buffer.push(ch);
                    buffer
                });

            let position = renderer.measure_width(
                &(before_preedit_text + &text),
                size,
                font.clone(),
            );

            if let Some(ime_state) = state.ime_state.as_mut() {
                ime_state.set_event(text, range);
            } else {
                let mut new_state = IMEState::default();
                new_state.set_event(text, range);
                let _ = state.ime_state.replace(new_state);
            }

            let position = (
                (text_bounds.x + position) as i32,
                (text_bounds.y) as i32 + size as i32,
            );
            ime.set_ime_position(position.0, position.1);

            return event::Status::Captured;
        }
        _ => {}
    }

    event::Status::Ignored
}

/// Draws the [`TextInput`] with the given [`Renderer`], overriding its
/// [`Value`] if provided.
///
/// [`Renderer`]: text::Renderer
pub fn draw<Renderer>(
    renderer: &mut Renderer,
    theme: &Renderer::Theme,
    layout: Layout<'_>,
    cursor_position: Point,
    state: &State,
    value: &Value,
    placeholder: &str,
    size: Option<f32>,
    font: &Renderer::Font,
    is_disabled: bool,
    is_secure: bool,
    icon: Option<&Icon<Renderer::Font>>,
    style: &<Renderer::Theme as StyleSheet>::Style,
) where
    Renderer: text::Renderer,
    Renderer::Theme: StyleSheet,
{
    let secure_value = is_secure.then(|| value.secure());
    let value = secure_value.as_ref().unwrap_or(value);

    let bounds = layout.bounds();

    let mut children_layout = layout.children();
    let text_bounds = children_layout.next().unwrap().bounds();

    let is_mouse_over = bounds.contains(cursor_position);

    let appearance = if is_disabled {
        theme.disabled(style)
    } else if state.is_focused() {
        theme.focused(style)
    } else if is_mouse_over {
        theme.hovered(style)
    } else {
        theme.active(style)
    };

    renderer.fill_quad(
        renderer::Quad {
            bounds,
            border_radius: appearance.border_radius.into(),
            border_width: appearance.border_width,
            border_color: appearance.border_color,
        },
        appearance.background,
    );

    if let Some(icon) = icon {
        let icon_layout = children_layout.next().unwrap();

        renderer.fill_text(Text {
            content: &icon.code_point.to_string(),
            size: icon.size.unwrap_or_else(|| renderer.default_size()),
            font: icon.font.clone(),
            color: appearance.icon_color,
            bounds: Rectangle {
                y: text_bounds.center_y(),
                ..icon_layout.bounds()
            },
            horizontal_alignment: alignment::Horizontal::Left,
            vertical_alignment: alignment::Vertical::Center,
        });
    }

    let text = value.to_string();
    let size = size.unwrap_or_else(|| renderer.default_size());

    let color = if text.is_empty()
        && state
            .ime_state
            .as_ref()
            .map_or(true, |state| state.preedit_text().is_empty())
    {
        theme.placeholder_color(style)
    } else if is_disabled {
        theme.disabled_color(style)
    } else {
        theme.value_color(style)
    };

    let (render_text, before_preedit_text) = if let Some(ime_state) =
        state.ime_state.as_ref()
    {
        let mut text = text;
        let chars_from_left = state.cursor.start(value);
        let before_preedit_text = text.chars().take(chars_from_left).fold(
            String::new(),
            |mut buffer, ch| {
                buffer.push(ch);
                buffer
            },
        );

        text.insert_str(before_preedit_text.len(), ime_state.preedit_text());

        (text, before_preedit_text)
    } else if text.is_empty() {
        (placeholder.to_owned(), String::new())
    } else {
        (text.clone(), String::new())
    };
    // display preedit text and exist text concated.
    // cursor position need to concated also.

    let (preedit_cursor_index, preedit_value) =
        if let Some(ime_state) = state.ime_state.as_ref() {
            (
                ime_state.before_cursor_text().chars().count(),
                Value::new(ime_state.preedit_text()),
            )
        } else {
            (0, Value::new(""))
        };

    let mut value = value.clone();

    value.insert_many(state.cursor.start(&value), preedit_value);

    let text_width = renderer.measure_width(&render_text, size, font.clone());

    let (cursor, offset) = if let Some(focus) = &state.is_focused {
        match state.cursor.state(&value) {
            cursor::State::Index(position) => {
                let (text_value_width, offset) =
                    measure_cursor_and_scroll_offset(
                        renderer,
                        text_bounds,
                        &value,
                        size,
                        position + preedit_cursor_index,
                        font.clone(),
                    );

                let is_cursor_visible = ((focus.now - focus.updated_at)
                    .as_millis()
                    / CURSOR_BLINK_INTERVAL_MILLIS)
                    % 2
                    == 0;

                let cursor = if is_cursor_visible {
                    Some((
                        renderer::Quad {
                            bounds: Rectangle {
                                x: text_bounds.x + text_value_width,
                                y: text_bounds.y,
                                width: 1.0,
                                height: text_bounds.height,
                            },
                            border_radius: 0.0.into(),
                            border_width: 0.0,
                            border_color: Color::TRANSPARENT,
                        },
                        theme.value_color(style),
                    ))
                } else {
                    None
                };

                (cursor, offset)
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
                        left + preedit_cursor_index,
                        font.clone(),
                    );

                let (right_position, right_offset) =
                    measure_cursor_and_scroll_offset(
                        renderer,
                        text_bounds,
                        &value,
                        size,
                        right + preedit_cursor_index,
                        font.clone(),
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
                            border_radius: 0.0.into(),
                            border_width: 0.0,
                            border_color: Color::TRANSPARENT,
                        },
                        theme.selection_color(style),
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

    let render = |renderer: &mut Renderer| {
        // ime mode should not paint selection.

        let fill_text = Text {
            content: &render_text,
            color,
            font: font.clone(),
            bounds: Rectangle {
                y: text_bounds.center_y(),
                width: f32::INFINITY,
                ..text_bounds
            },
            size,
            horizontal_alignment: alignment::Horizontal::Left,
            vertical_alignment: alignment::Vertical::Center,
        };

        if let Some(ime_state) = state.ime_state.as_ref() {
            // draw under line.

            let before_preedit_text_width = renderer.measure_width(
                &before_preedit_text,
                size,
                font.clone(),
            );

            let splits = ime_state.split_to_pieces();

            let _ = splits.iter().enumerate().fold(
                text_bounds.x + before_preedit_text_width,
                |offset, (idx, t)| {
                    if let Some(t) = t {
                        let width =
                            renderer.measure_width(t, size, font.clone());
                        let quad = renderer::Quad {
                            bounds: Rectangle {
                                x: offset,
                                y: if idx == 1 {
                                    text_bounds.y + size - 2.0
                                } else {
                                    text_bounds.y + size
                                },
                                width,
                                height: if idx == 1 { 3.0 } else { 1.0 },
                            },
                            border_radius: 0.0.into(),
                            border_width: 0.0,
                            border_color: Color::default(),
                        };
                        renderer.fill_quad(quad, theme.value_color(style));
                        width + offset
                    } else {
                        offset
                    }
                },
            );
        }

        if let Some((cursor, color)) = cursor {
            renderer.fill_quad(cursor, color);
        }
        renderer.fill_text(fill_text);
    };

    if text_width > text_bounds.width {
        renderer.with_layer(text_bounds, |renderer| {
            renderer.with_translation(Vector::new(-offset, 0.0), render)
        });
    } else {
        render(renderer);
    }
}

/// Computes the current [`mouse::Interaction`] of the [`TextInput`].
pub fn mouse_interaction(
    layout: Layout<'_>,
    cursor_position: Point,
    is_disabled: bool,
) -> mouse::Interaction {
    if layout.bounds().contains(cursor_position) {
        if is_disabled {
            mouse::Interaction::NotAllowed
        } else {
            mouse::Interaction::Text
        }
    } else {
        mouse::Interaction::default()
    }
}

/// The state of a [`TextInput`].
#[derive(Debug, Default, Clone)]
pub struct State {
    is_focused: Option<Focus>,
    is_dragging: bool,
    is_pasting: Option<Value>,
    last_click: Option<mouse::Click>,
    cursor: Cursor,
    keyboard_modifiers: keyboard::Modifiers,
    ime_state: Option<IMEState>,
    // TODO: Add stateful horizontal scrolling offset
}

#[derive(Debug, Clone, Copy)]
struct Focus {
    updated_at: Instant,
    now: Instant,
}

impl State {
    /// Creates a new [`State`], representing an unfocused [`TextInput`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a new [`State`], representing a focused [`TextInput`].
    pub fn focused() -> Self {
        Self {
            is_focused: None,
            is_dragging: false,
            is_pasting: None,
            last_click: None,
            cursor: Cursor::default(),
            keyboard_modifiers: keyboard::Modifiers::default(),
            ime_state: None,
        }
    }

    /// Returns whether the [`TextInput`] is currently focused or not.
    pub fn is_focused(&self) -> bool {
        self.is_focused.is_some()
    }

    /// Returns the [`Cursor`] of the [`TextInput`].
    pub fn cursor(&self) -> Cursor {
        self.cursor
    }

    /// Focuses the [`TextInput`].
    pub fn focus(&mut self) {
        let now = Instant::now();

        self.is_focused = Some(Focus {
            updated_at: now,
            now,
        });

        self.move_cursor_to_end();
    }

    /// Unfocuses the [`TextInput`].
    pub fn unfocus(&mut self) {
        self.is_focused = None;
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

impl operation::Focusable for State {
    fn is_focused(&self) -> bool {
        State::is_focused(self)
    }

    fn focus(&mut self) {
        State::focus(self)
    }

    fn unfocus(&mut self) {
        State::unfocus(self)
    }
}

impl operation::TextInput for State {
    fn move_cursor_to_front(&mut self) {
        State::move_cursor_to_front(self)
    }

    fn move_cursor_to_end(&mut self) {
        State::move_cursor_to_end(self)
    }

    fn move_cursor_to(&mut self, position: usize) {
        State::move_cursor_to(self, position)
    }

    fn select_all(&mut self) {
        State::select_all(self)
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
    size: f32,
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
    size: f32,
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
    size: Option<f32>,
    value: &Value,
    state: &State,
    x: f32,
) -> Option<usize>
where
    Renderer: text::Renderer,
{
    let size = size.unwrap_or_else(|| renderer.default_size());

    let offset =
        offset(renderer, text_bounds, font.clone(), size, value, state);

    renderer
        .hit_test(
            &value.to_string(),
            size,
            font,
            Size::INFINITY,
            Point::new(x + offset, text_bounds.height / 2.0),
            true,
        )
        .map(text::Hit::cursor)
}

const CURSOR_BLINK_INTERVAL_MILLIS: u128 = 500;
