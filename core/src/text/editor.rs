//! Edit text.
use crate::clipboard;
use crate::input_method;
use crate::keyboard;
use crate::keyboard::key;
use crate::mouse;
use crate::renderer;
use crate::text::highlighter::{self, Highlighter};
use crate::text::{self, Alignment, LineHeight, Position, Wrapping};
use crate::time::{Duration, Instant};
use crate::widget::operation::{Focusable, TextInput};
use crate::window;
use crate::{Color, Event, InputMethod, Padding, Pixels, Point, Rectangle, Size, SmolStr, Vector};

use std::borrow::Cow;
use std::sync::Arc;

/// A component that can be used by widgets to edit multi-line text.
pub trait Editor: Sized + Default {
    /// The font of the [`Editor`].
    type Font: Copy + PartialEq + Default;

    /// Creates a new [`Editor`] laid out with the given text.
    fn with_text(text: &str) -> Self;

    /// Returns true if the [`Editor`] has no contents.
    fn is_empty(&self) -> bool;

    /// Returns the current [`Cursor`] of the [`Editor`].
    fn cursor(&self) -> Cursor;

    /// Returns the current [`Selection`] of the [`Editor`].
    fn selection(&self) -> Selection;

    /// Returns the current selected text of the [`Editor`].
    fn copy(&self) -> Option<String>;

    /// Returns the text of the given line in the [`Editor`], if it exists.
    fn line(&self, index: usize) -> Option<Line<'_>>;

    /// Returns the amount of lines in the [`Editor`].
    fn line_count(&self) -> usize;

    /// Performs an [`Action`] on the [`Editor`].
    fn perform(&mut self, action: Action);

    /// Moves the cursor to the given position.
    fn move_to(&mut self, cursor: Cursor);

    /// Returns the current boundaries of the [`Editor`].
    fn bounds(&self) -> Size;

    /// Returns the minimum boundaries to fit the current contents of
    /// the [`Editor`].
    fn min_bounds(&self) -> Size;

    /// Returns the hint factor of the [`Editor`].
    fn hint_factor(&self) -> Option<f32>;

    /// Updates the [`Editor`] with some new attributes.
    fn update(
        &mut self,
        new_bounds: Size,
        new_font: Self::Font,
        new_size: Pixels,
        new_line_height: LineHeight,
        new_wrapping: Wrapping,
        new_alignment: Alignment,
        new_hint_factor: Option<f32>,
        new_highlighter: &mut impl Highlighter,
    );

    /// Overwrites the current contents of the [`Editor`].
    fn overwrite(&mut self, new_text: &str);

    /// Runs a text [`Highlighter`] in the [`Editor`].
    fn highlight<H: Highlighter>(
        &mut self,
        font: Self::Font,
        highlighter: &mut H,
        format_highlight: impl Fn(&H::Highlight) -> highlighter::Format<Self::Font>,
    );

    /// Returns an iterator of the text of the lines in the [`Editor`].
    fn lines(&self) -> impl Iterator<Item = Line<'_>> {
        (0..)
            .map(|i| self.line(i))
            .take_while(Option::is_some)
            .flatten()
    }

    /// Returns the text of the [`Editor`].
    fn text(&self) -> String {
        let mut contents = String::new();
        let mut lines = self.lines().peekable();

        while let Some(line) = lines.next() {
            contents.push_str(&line.text);

            if lines.peek().is_some() {
                contents.push_str(if line.ending == LineEnding::None {
                    LineEnding::default().as_str()
                } else {
                    line.ending.as_str()
                });
            }
        }

        contents
    }

    /// Returns the current [`Font`](Self::Font) of the [`Editor`].
    fn font(&self) -> Self::Font;

    /// Returns the current text size of the [`Editor`].
    fn text_size(&self) -> Pixels;

    /// Returns the current [`LineHeight`] of the [`Editor`].
    fn line_height(&self) -> LineHeight;
}

/// An interaction with an [`Editor`].
#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    /// Apply a [`Motion`].
    Move(Motion),
    /// Select text with a given [`Motion`].
    Select(Motion),
    /// Select the word at the current cursor.
    SelectWord,
    /// Select the line at the current cursor.
    SelectLine,
    /// Select the entire buffer.
    SelectAll,
    /// Perform an [`Edit`].
    Edit(Edit),
    /// Click the [`Editor`] at the given [`Point`].
    Click(Point),
    /// Drag the mouse on the [`Editor`] to the given [`Point`].
    Drag(Point),
    /// Scroll the [`Editor`] a certain amount of lines.
    Scroll {
        /// The amount of lines to scroll.
        lines: i32,
    },
}

impl Action {
    /// Returns whether the [`Action`] is an editing action.
    pub fn is_edit(&self) -> bool {
        matches!(self, Self::Edit(_))
    }
}

/// An action that edits text.
#[derive(Debug, Clone, PartialEq)]
pub enum Edit {
    /// Insert the given character.
    Insert(char),
    /// Paste the given text.
    Paste(Arc<String>),
    /// Break the current line.
    Enter,
    /// Indent the current line.
    Indent,
    /// Unindent the current line.
    Unindent,
    /// Delete the previous character.
    Backspace,
    /// Delete the next character.
    Delete,
    /// Undo the last change performed on the [`Editor`].
    Undo,
    /// Redo the last undone change on the [`Editor`].
    Redo,
}

/// A cursor movement.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Motion {
    /// Move left.
    Left,
    /// Move right.
    Right,
    /// Move up.
    Up,
    /// Move down.
    Down,
    /// Move to the left boundary of a word.
    WordLeft,
    /// Move to the right boundary of a word.
    WordRight,
    /// Move to the start of the line.
    Home,
    /// Move to the end of the line.
    End,
    /// Move to the start of the previous window.
    PageUp,
    /// Move to the start of the next window.
    PageDown,
    /// Move to the start of the text.
    DocumentStart,
    /// Move to the end of the text.
    DocumentEnd,
}

impl Motion {
    /// Widens the [`Motion`], if possible.
    pub fn widen(self) -> Self {
        match self {
            Self::Left => Self::WordLeft,
            Self::Right => Self::WordRight,
            Self::Home => Self::DocumentStart,
            Self::End => Self::DocumentEnd,
            _ => self,
        }
    }

    /// Returns the [`Direction`] of the [`Motion`].
    pub fn direction(&self) -> Direction {
        match self {
            Self::Left
            | Self::Up
            | Self::WordLeft
            | Self::Home
            | Self::PageUp
            | Self::DocumentStart => Direction::Left,
            Self::Right
            | Self::Down
            | Self::WordRight
            | Self::End
            | Self::PageDown
            | Self::DocumentEnd => Direction::Right,
        }
    }
}

/// A direction in some text.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    /// <-
    Left,
    /// ->
    Right,
}

/// The cursor of an [`Editor`].
#[derive(Debug, Clone)]
pub enum Selection {
    /// Cursor without a selection
    Caret(Point),

    /// Cursor selecting a range of text
    Range(Vec<Rectangle>),
}

/// The range of an [`Editor`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Cursor {
    /// The cursor position.
    pub position: Position,

    /// The selection position, if any.
    pub selection: Option<Position>,
}

/// A line of an [`Editor`].
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Line<'a> {
    /// The raw text of the [`Line`].
    pub text: Cow<'a, str>,
    /// The line ending of the [`Line`].
    pub ending: LineEnding,
}

/// The line ending of a [`Line`].
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum LineEnding {
    /// Use `\n` for line ending (POSIX-style)
    #[default]
    Lf,
    /// Use `\r\n` for line ending (Windows-style)
    CrLf,
    /// Use `\r` for line ending (many legacy systems)
    Cr,
    /// Use `\n\r` for line ending (some legacy systems)
    LfCr,
    /// No line ending
    None,
}

impl LineEnding {
    /// Gets the string representation of the [`LineEnding`].
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Lf => "\n",
            Self::CrLf => "\r\n",
            Self::Cr => "\r",
            Self::LfCr => "\n\r",
            Self::None => "",
        }
    }
}

/// The internal state of an [`Editor`].
#[derive(Debug, Clone, Default)]
pub struct State {
    focus: Option<Focus>,
    preedit: Option<input_method::Preedit>,
    last_click: Option<mouse::Click>,
    drag_click: Option<mouse::click::Kind>,
    partial_scroll: f32,
}

impl State {
    /// Creates a new [`State`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Updates the [`State`] for the given [`Editor`] and returns any relevant [`Update`].
    pub fn update<Message>(
        &mut self,
        editor: &impl Editor,
        event: &Event,
        bounds: Rectangle,
        padding: Padding,
        cursor: mouse::Cursor,
        key_binding: impl Fn(KeyPress) -> Option<Binding<Message>>,
    ) -> Option<Update<Message>> {
        match event {
            Event::Window(window::Event::Unfocused) => {
                if let Some(focus) = &mut self.focus {
                    focus.is_window_focused = false;
                }

                None
            }
            Event::Window(window::Event::Focused) => {
                if let Some(focus) = &mut self.focus {
                    focus.is_window_focused = true;
                    focus.updated_at = Instant::now();
                }

                Some(Update::Focus)
            }
            Event::Window(window::Event::RedrawRequested(now)) => {
                let focus = self.focus.as_mut()?;

                if !focus.is_window_focused {
                    return None;
                }

                focus.now = *now;

                let millis_until_redraw = Focus::CURSOR_BLINK_INTERVAL_MILLIS
                    - (focus.now - focus.updated_at).as_millis()
                        % Focus::CURSOR_BLINK_INTERVAL_MILLIS;

                Some(Update::RedrawAt(
                    focus.now + Duration::from_millis(millis_until_redraw as u64),
                ))
            }
            Event::Clipboard(clipboard::Event::Read(Ok(content))) => {
                let focus = self.focus.as_ref()?;

                if !focus.is_window_focused {
                    return None;
                }

                let clipboard::Content::Text(text) = content.as_ref() else {
                    return None;
                };

                Some(Update::Action(Action::Edit(Edit::Paste(Arc::new(
                    text.clone(),
                )))))
            }
            Event::Mouse(event) => match event {
                mouse::Event::ButtonPressed(mouse::Button::Left) => {
                    if let Some(cursor_position) = cursor.position_in(bounds) {
                        let cursor_position =
                            cursor_position - Vector::new(padding.left, padding.top);

                        let click = mouse::Click::new(
                            cursor_position,
                            mouse::Button::Left,
                            self.last_click,
                        );

                        let action = match click.kind() {
                            mouse::click::Kind::Single => Action::Click(click.position()),
                            mouse::click::Kind::Double => Action::SelectWord,
                            mouse::click::Kind::Triple => Action::SelectLine,
                        };

                        self.focus = Some(Focus::now());
                        self.last_click = Some(click);
                        self.drag_click = Some(click.kind());

                        Some(Update::Action(action))
                    } else if self.focus.is_some() {
                        self.focus = None;

                        Some(Update::Unfocus)
                    } else {
                        None
                    }
                }
                mouse::Event::ButtonReleased(mouse::Button::Left) => {
                    self.drag_click = None;

                    Some(Update::Release)
                }
                mouse::Event::CursorMoved { .. } => match self.drag_click {
                    Some(mouse::click::Kind::Single) => {
                        let position =
                            cursor.position_in(bounds)? - Vector::new(padding.left, padding.top);

                        Some(Update::Action(Action::Drag(position)))
                    }
                    _ => None,
                },
                mouse::Event::WheelScrolled { delta } if cursor.is_over(bounds) => {
                    let bounds = editor.bounds();

                    if bounds.height >= i32::MAX as f32 {
                        return None;
                    }

                    let lines = match delta {
                        mouse::ScrollDelta::Lines { y, .. } => {
                            if y.abs() > 0.0 {
                                y.signum() * -(y.abs() * 4.0).max(1.0)
                            } else {
                                0.0
                            }
                        }
                        mouse::ScrollDelta::Pixels { y, .. } => -y / 4.0,
                    };

                    let lines = lines + self.partial_scroll;
                    self.partial_scroll = lines.fract();

                    Some(Update::Action(Action::Scroll {
                        lines: lines as i32,
                    }))
                }
                _ => None,
            },
            Event::InputMethod(event) => match event {
                input_method::Event::Opened | input_method::Event::Closed => {
                    let is_open = matches!(event, input_method::Event::Opened);
                    self.preedit = is_open.then(input_method::Preedit::new);

                    Some(Update::InputMethod)
                }
                input_method::Event::Preedit(content, selection) if self.focus.is_some() => {
                    self.preedit = Some(input_method::Preedit {
                        content: content.clone(),
                        selection: selection.clone(),
                        text_size: Some(editor.text_size()),
                    });

                    Some(Update::InputMethod)
                }
                input_method::Event::Commit(content) if self.focus.is_some() => Some(
                    Update::Action(Action::Edit(Edit::Paste(Arc::new(content.clone())))),
                ),
                _ => None,
            },
            Event::Keyboard(keyboard::Event::KeyPressed {
                key,
                modified_key,
                physical_key,
                modifiers,
                text,
                ..
            }) => {
                let key_press = KeyPress {
                    key: key.clone(),
                    modified_key: modified_key.clone(),
                    physical_key: *physical_key,
                    modifiers: *modifiers,
                    text: text.clone(),
                    is_focused: self.is_focused(),
                };

                fn apply_binding<Message>(
                    binding: Binding<Message>,
                    editor: &impl Editor,
                    state: &mut State,
                ) -> Option<Update<Message>> {
                    let action = |action| Update::Action(action);
                    let edit = |edit| action(Action::Edit(edit));

                    match binding {
                        Binding::Unfocus => {
                            state.focus = None;
                            state.drag_click = None;

                            None
                        }
                        Binding::Copy => {
                            let selection = editor.copy()?;

                            Some(Update::Copy(selection))
                        }
                        Binding::Cut => {
                            let selection = editor.copy()?;

                            Some(Update::Sequence(vec![
                                Update::Copy(selection),
                                edit(Edit::Backspace),
                            ]))
                        }
                        Binding::Paste => Some(Update::Paste),
                        Binding::Undo => Some(edit(Edit::Undo)),
                        Binding::Redo => Some(edit(Edit::Redo)),
                        Binding::Move(motion) => Some(action(Action::Move(motion))),
                        Binding::Select(motion) => Some(action(Action::Select(motion))),
                        Binding::SelectWord => Some(action(Action::SelectWord)),
                        Binding::SelectLine => Some(action(Action::SelectLine)),
                        Binding::SelectAll => Some(action(Action::SelectAll)),
                        Binding::Insert(c) => Some(action(Action::Edit(Edit::Insert(c)))),
                        Binding::Enter => Some(action(Action::Edit(Edit::Enter))),
                        Binding::Backspace => Some(action(Action::Edit(Edit::Backspace))),
                        Binding::Delete => Some(action(Action::Edit(Edit::Delete))),
                        Binding::Sequence(sequence) => {
                            let updates: Vec<_> = sequence
                                .into_iter()
                                .flat_map(|binding| apply_binding(binding, editor, state))
                                .collect();

                            if updates.is_empty() {
                                return None;
                            }

                            Some(Update::Sequence(updates))
                        }
                        Binding::Custom(message) => Some(Update::Custom(message)),
                    }
                }

                let update = apply_binding(key_binding(key_press)?, editor, self);

                if let Some(focus) = &mut self.focus {
                    focus.updated_at = Instant::now();
                }

                update
            }
            _ => None,
        }
    }

    /// Returns the current [`InputMethod`] of the [`State`] for the given [`Editor`].
    pub fn input_method<'a>(
        &'a self,
        editor: &impl Editor,
        position: Point,
    ) -> InputMethod<&'a str> {
        let Some(Focus {
            is_window_focused: true,
            ..
        }) = &self.focus
        else {
            return InputMethod::Disabled;
        };

        let translation = position - Point::ORIGIN;

        let cursor = match editor.selection() {
            Selection::Caret(position) => position,
            Selection::Range(ranges) => ranges.first().cloned().unwrap_or_default().position(),
        };

        let line_height = editor.line_height().to_absolute(editor.text_size());

        let position = cursor + translation;

        InputMethod::Enabled {
            cursor: Rectangle::new(position, Size::new(1.0, f32::from(line_height))),
            purpose: input_method::Purpose::Normal,
            preedit: self.preedit.as_ref().map(input_method::Preedit::as_ref),
        }
    }

    /// Draws the given [`Editor`] with the current [`State`].
    pub fn draw<Renderer: text::Renderer>(
        &self,
        editor: &Renderer::Editor,
        renderer: &mut Renderer,
        position: Point,
        clip_bounds: Rectangle,
        style: Style,
    ) {
        let bounds = Rectangle::new(position, editor.bounds());

        let Some(clip_bounds) = clip_bounds.intersection(&bounds) else {
            return;
        };

        if !editor.is_empty() {
            renderer.fill_editor(editor, position, style.value, clip_bounds);
        }

        if !self.is_focused() {
            return;
        }

        let translation = position - Point::ORIGIN;
        let text_size = editor.text_size();
        let line_height = editor.line_height();

        match editor.selection() {
            Selection::Caret(position) if self.is_cursor_visible() => {
                let cursor = Rectangle::new(
                    position + translation,
                    Size::new(
                        if renderer::CRISP {
                            (1.0 / renderer.hint_factor().unwrap_or(1.0)).max(1.0)
                        } else {
                            1.0
                        },
                        line_height.to_absolute(text_size).into(),
                    ),
                );

                if let Some(clipped_cursor) = clip_bounds.intersection(&cursor) {
                    renderer.fill_quad(
                        renderer::Quad {
                            bounds: clipped_cursor,
                            ..renderer::Quad::default()
                        },
                        style.value,
                    );
                }
            }
            Selection::Range(ranges) => {
                for range in ranges
                    .into_iter()
                    .filter_map(|range| clip_bounds.intersection(&(range + translation)))
                {
                    renderer.fill_quad(
                        renderer::Quad {
                            bounds: range.round(),
                            ..renderer::Quad::default()
                        },
                        style.selection,
                    );
                }
            }
            Selection::Caret(_) => {
                // Drawing an empty quad helps some renderers to track the damage of the blinking cursor
                renderer.fill_quad(renderer::Quad::default(), Color::TRANSPARENT);
            }
        }
    }

    /// Returns whether the cursor of the [`Editor`] is visible.
    pub fn is_cursor_visible(&self) -> bool {
        self.focus.as_ref().is_some_and(Focus::is_cursor_visible)
    }
}

/// The visual style of an [`Editor`].
pub struct Style {
    /// The [`Color`] of the contents.
    pub value: Color,

    /// The background [`Color`] of any selection.
    pub selection: Color,
}

#[derive(Debug, Clone)]
struct Focus {
    updated_at: Instant,
    now: Instant,
    is_window_focused: bool,
}

impl Focus {
    const CURSOR_BLINK_INTERVAL_MILLIS: u128 = 500;

    fn now() -> Self {
        let now = Instant::now();

        Self {
            updated_at: now,
            now,
            is_window_focused: true,
        }
    }

    fn is_cursor_visible(&self) -> bool {
        self.is_window_focused
            && ((self.now - self.updated_at).as_millis() / Self::CURSOR_BLINK_INTERVAL_MILLIS)
                .is_multiple_of(2)
    }
}

impl State {
    /// Returns whether the [`Editor`] is currently focused or not.
    pub fn is_focused(&self) -> bool {
        self.focus.is_some()
    }
}

impl Focusable for State {
    fn is_focused(&self) -> bool {
        self.focus.is_some()
    }

    fn focus(&mut self) {
        self.focus = Some(Focus::now());
    }

    fn unfocus(&mut self) {
        self.focus = None;
    }
}

/// A binding to an action in the [`Editor`].
#[derive(Debug, Clone, PartialEq)]
pub enum Binding<Message> {
    /// Unfocus the [`Editor`].
    Unfocus,
    /// Copy the selection of the [`Editor`].
    Copy,
    /// Cut the selection of the [`Editor`].
    Cut,
    /// Paste the clipboard contents in the [`Editor`].
    Paste,
    /// Undo the last change peformed in the [`Editor`].
    Undo,
    /// Redo the last change undone in the [`Editor`].
    Redo,
    /// Apply a [`Motion`].
    Move(Motion),
    /// Select text with a given [`Motion`].
    Select(Motion),
    /// Select the word at the current cursor.
    SelectWord,
    /// Select the line at the current cursor.
    SelectLine,
    /// Select the entire buffer.
    SelectAll,
    /// Insert the given character.
    Insert(char),
    /// Break the current line.
    Enter,
    /// Delete the previous character.
    Backspace,
    /// Delete the next character.
    Delete,
    /// A sequence of bindings to execute.
    Sequence(Vec<Self>),
    /// Produce the given message.
    Custom(Message),
}

/// A key press.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyPress {
    /// The original key pressed without modifiers applied to it.
    ///
    /// You should use this key for combinations (e.g. Ctrl+C).
    pub key: keyboard::Key,
    /// The key pressed with modifiers applied to it.
    ///
    /// You should use this key for any single key bindings (e.g. motions).
    pub modified_key: keyboard::Key,
    /// The physical key pressed.
    ///
    /// You should use this key for layout-independent bindings.
    pub physical_key: keyboard::key::Physical,
    /// The state of the keyboard modifiers.
    pub modifiers: keyboard::Modifiers,
    /// The text produced by the key press.
    pub text: Option<SmolStr>,
    /// Whether the [`Editor`] is focused.
    pub is_focused: bool,
}

impl<Message> Binding<Message> {
    /// Returns the default [`Binding`] for the given key press.
    pub fn from_key_press(event: KeyPress) -> Option<Self> {
        let KeyPress {
            key,
            modified_key,
            physical_key,
            modifiers,
            text,
            is_focused,
        } = event;

        if !is_focused {
            return None;
        }

        let combination = match key.to_latin(physical_key) {
            Some('c') if modifiers.command() => Some(Self::Copy),
            Some('x') if modifiers.command() => Some(Self::Cut),
            Some('v') if modifiers.command() && !modifiers.alt() => Some(Self::Paste),
            Some('a') if modifiers.command() => Some(Self::SelectAll),
            Some('z') if modifiers.command() => Some(Self::Undo),
            Some('y') if modifiers.command() => Some(Self::Redo),
            _ => None,
        };

        if let Some(binding) = combination {
            return Some(binding);
        }

        #[cfg(target_os = "macos")]
        let modified_key = convert_macos_shortcut(&key, modifiers).unwrap_or(modified_key);

        match modified_key.as_ref() {
            keyboard::Key::Named(key::Named::Enter) => Some(Self::Enter),
            keyboard::Key::Named(key::Named::Backspace) => Some(Self::Backspace),
            keyboard::Key::Named(key::Named::Delete)
                if text.is_none() || text.as_deref() == Some("\u{7f}") =>
            {
                Some(Self::Delete)
            }
            keyboard::Key::Named(key::Named::Escape) => Some(Self::Unfocus),
            _ => {
                if let Some(text) = text {
                    let c = text.chars().find(|c| !c.is_control())?;

                    Some(Self::Insert(c))
                } else if let keyboard::Key::Named(named_key) = key.as_ref() {
                    let motion = motion(named_key)?;

                    let motion = if modifiers.macos_command() {
                        match motion {
                            Motion::Left => Motion::Home,
                            Motion::Right => Motion::End,
                            _ => motion,
                        }
                    } else {
                        motion
                    };

                    let motion = if modifiers.jump() {
                        motion.widen()
                    } else {
                        motion
                    };

                    Some(if modifiers.shift() {
                        Self::Select(motion)
                    } else {
                        Self::Move(motion)
                    })
                } else {
                    None
                }
            }
        }
    }
}

/// The update of an [`Editor`], returned by [`State::update`].
pub enum Update<Message> {
    /// An [`Action`] must be performed in the [`Editor`].
    Action(Action),
    /// The [`Editor`] just gained focus.
    Focus,
    /// The [`Editor`] just lost focus.
    Unfocus,
    /// The [`Editor`] changed its [`InputMethod`].
    InputMethod,
    /// A mouse press was released in the [`Editor`].
    Release,
    /// The [`Editor`] must copy some text to the clipboard.
    Copy(String),
    /// The [`Editor`] must paste the clipboard contents.
    Paste,
    /// The [`Editor`] must be redrawn at the given [`Instant`].
    RedrawAt(Instant),
    /// The [`Editor`] produced a custom message that must be published.
    Custom(Message),
    /// The [`Editor`] produced a sequence of updates.
    Sequence(Vec<Update<Message>>),
}

fn motion(key: key::Named) -> Option<Motion> {
    match key {
        key::Named::ArrowLeft => Some(Motion::Left),
        key::Named::ArrowRight => Some(Motion::Right),
        key::Named::ArrowUp => Some(Motion::Up),
        key::Named::ArrowDown => Some(Motion::Down),
        key::Named::Home => Some(Motion::Home),
        key::Named::End => Some(Motion::End),
        key::Named::PageUp => Some(Motion::PageUp),
        key::Named::PageDown => Some(Motion::PageDown),
        _ => None,
    }
}

#[cfg(target_os = "macos")]
fn convert_macos_shortcut(
    key: &keyboard::Key,
    modifiers: keyboard::Modifiers,
) -> Option<keyboard::Key> {
    if modifiers != keyboard::Modifiers::CTRL {
        return None;
    }

    let key = match key.as_ref() {
        keyboard::Key::Character("b") => key::Named::ArrowLeft,
        keyboard::Key::Character("f") => key::Named::ArrowRight,
        keyboard::Key::Character("a") => key::Named::Home,
        keyboard::Key::Character("e") => key::Named::End,
        keyboard::Key::Character("h") => key::Named::Backspace,
        keyboard::Key::Character("d") => key::Named::Delete,
        _ => return None,
    };

    Some(keyboard::Key::Named(key))
}

impl<T: Editor> TextInput for T {
    fn text(&self) -> text::Fragment<'_> {
        text::Fragment::Owned(Editor::text(self))
    }

    fn move_cursor_to_front(&mut self) {
        self.perform(Action::Move(Motion::DocumentStart));
    }

    fn move_cursor_to_end(&mut self) {
        self.perform(Action::Move(Motion::DocumentEnd));
    }

    fn move_cursor_to(&mut self, position: text::Position) {
        self.move_to(Cursor {
            position,
            selection: None,
        });
    }

    fn select_all(&mut self) {
        self.perform(Action::SelectAll);
    }

    fn select_range(&mut self, start: text::Position, end: text::Position) {
        self.move_to(Cursor {
            position: start,
            selection: Some(end),
        });
    }
}
