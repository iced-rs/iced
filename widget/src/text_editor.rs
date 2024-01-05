//! Display a multi-line text input for text editing.
use crate::core::event::{self, Event};
use crate::core::keyboard;
use crate::core::layout::{self, Layout};
use crate::core::mouse;
use crate::core::renderer;
use crate::core::text::editor::{Cursor, Editor as _};
use crate::core::text::highlighter::{self, Highlighter};
use crate::core::text::{self, LineHeight};
use crate::core::widget::{self, Widget};
use crate::core::{
    Clipboard, Color, Element, Length, Padding, Pixels, Rectangle, Shell, Size,
    Vector,
};

use std::cell::RefCell;
use std::fmt;
use std::ops::DerefMut;
use std::sync::Arc;

pub use crate::style::text_editor::{Appearance, StyleSheet};
pub use text::editor::{Action, Edit, Motion};

/// A multi-line text input.
#[allow(missing_debug_implementations)]
pub struct TextEditor<'a, Highlighter, Message, Renderer = crate::Renderer>
where
    Highlighter: text::Highlighter,
    Renderer: text::Renderer,
    Renderer::Theme: StyleSheet,
{
    content: &'a Content<Renderer>,
    font: Option<Renderer::Font>,
    text_size: Option<Pixels>,
    line_height: LineHeight,
    width: Length,
    height: Length,
    padding: Padding,
    style: <Renderer::Theme as StyleSheet>::Style,
    on_edit: Option<Box<dyn Fn(Action) -> Message + 'a>>,
    highlighter_settings: Highlighter::Settings,
    highlighter_format: fn(
        &Highlighter::Highlight,
        &Renderer::Theme,
    ) -> highlighter::Format<Renderer::Font>,
}

impl<'a, Message, Renderer>
    TextEditor<'a, highlighter::PlainText, Message, Renderer>
where
    Renderer: text::Renderer,
    Renderer::Theme: StyleSheet,
{
    /// Creates new [`TextEditor`] with the given [`Content`].
    pub fn new(content: &'a Content<Renderer>) -> Self {
        Self {
            content,
            font: None,
            text_size: None,
            line_height: LineHeight::default(),
            width: Length::Fill,
            height: Length::Fill,
            padding: Padding::new(5.0),
            style: Default::default(),
            on_edit: None,
            highlighter_settings: (),
            highlighter_format: |_highlight, _theme| {
                highlighter::Format::default()
            },
        }
    }
}

impl<'a, Highlighter, Message, Renderer>
    TextEditor<'a, Highlighter, Message, Renderer>
where
    Highlighter: text::Highlighter,
    Renderer: text::Renderer,
    Renderer::Theme: StyleSheet,
{
    /// Sets the message that should be produced when some action is performed in
    /// the [`TextEditor`].
    ///
    /// If this method is not called, the [`TextEditor`] will be disabled.
    pub fn on_action(
        mut self,
        on_edit: impl Fn(Action) -> Message + 'a,
    ) -> Self {
        self.on_edit = Some(Box::new(on_edit));
        self
    }

    /// Sets the [`Font`] of the [`TextEditor`].
    ///
    /// [`Font`]: text::Renderer::Font
    pub fn font(mut self, font: impl Into<Renderer::Font>) -> Self {
        self.font = Some(font.into());
        self
    }

    /// Sets the [`Padding`] of the [`TextEditor`].
    pub fn padding(mut self, padding: impl Into<Padding>) -> Self {
        self.padding = padding.into();
        self
    }

    /// Highlights the [`TextEditor`] with the given [`Highlighter`] and
    /// a strategy to turn its highlights into some text format.
    pub fn highlight<H: text::Highlighter>(
        self,
        settings: H::Settings,
        to_format: fn(
            &H::Highlight,
            &Renderer::Theme,
        ) -> highlighter::Format<Renderer::Font>,
    ) -> TextEditor<'a, H, Message, Renderer> {
        TextEditor {
            content: self.content,
            font: self.font,
            text_size: self.text_size,
            line_height: self.line_height,
            width: self.width,
            height: self.height,
            padding: self.padding,
            style: self.style,
            on_edit: self.on_edit,
            highlighter_settings: settings,
            highlighter_format: to_format,
        }
    }

    /// Sets the style of the [`TextEditor`].
    pub fn style(
        mut self,
        style: impl Into<<Renderer::Theme as StyleSheet>::Style>,
    ) -> Self {
        self.style = style.into();
        self
    }
}

/// The content of a [`TextEditor`].
pub struct Content<R = crate::Renderer>(RefCell<Internal<R>>)
where
    R: text::Renderer;

struct Internal<R>
where
    R: text::Renderer,
{
    editor: R::Editor,
    is_dirty: bool,
}

impl<R> Content<R>
where
    R: text::Renderer,
{
    /// Creates an empty [`Content`].
    pub fn new() -> Self {
        Self::with_text("")
    }

    /// Creates a [`Content`] with the given text.
    pub fn with_text(text: &str) -> Self {
        Self(RefCell::new(Internal {
            editor: R::Editor::with_text(text),
            is_dirty: true,
        }))
    }

    /// Performs an [`Action`] on the [`Content`].
    pub fn perform(&mut self, action: Action) {
        let internal = self.0.get_mut();

        internal.editor.perform(action);
        internal.is_dirty = true;
    }

    /// Returns the amount of lines of the [`Content`].
    pub fn line_count(&self) -> usize {
        self.0.borrow().editor.line_count()
    }

    /// Returns the text of the line at the given index, if it exists.
    pub fn line(
        &self,
        index: usize,
    ) -> Option<impl std::ops::Deref<Target = str> + '_> {
        std::cell::Ref::filter_map(self.0.borrow(), |internal| {
            internal.editor.line(index)
        })
        .ok()
    }

    /// Returns an iterator of the text of the lines in the [`Content`].
    pub fn lines(
        &self,
    ) -> impl Iterator<Item = impl std::ops::Deref<Target = str> + '_> {
        struct Lines<'a, Renderer: text::Renderer> {
            internal: std::cell::Ref<'a, Internal<Renderer>>,
            current: usize,
        }

        impl<'a, Renderer: text::Renderer> Iterator for Lines<'a, Renderer> {
            type Item = std::cell::Ref<'a, str>;

            fn next(&mut self) -> Option<Self::Item> {
                let line = std::cell::Ref::filter_map(
                    std::cell::Ref::clone(&self.internal),
                    |internal| internal.editor.line(self.current),
                )
                .ok()?;

                self.current += 1;

                Some(line)
            }
        }

        Lines {
            internal: self.0.borrow(),
            current: 0,
        }
    }

    /// Returns the text of the [`Content`].
    ///
    /// Lines are joined with `'\n'`.
    pub fn text(&self) -> String {
        let mut text = self.lines().enumerate().fold(
            String::new(),
            |mut contents, (i, line)| {
                if i > 0 {
                    contents.push('\n');
                }

                contents.push_str(&line);

                contents
            },
        );

        if !text.ends_with('\n') {
            text.push('\n');
        }

        text
    }

    /// Returns the selected text of the [`Content`].
    pub fn selection(&self) -> Option<String> {
        self.0.borrow().editor.selection()
    }

    /// Returns the current cursor position of the [`Content`].
    pub fn cursor_position(&self) -> (usize, usize) {
        self.0.borrow().editor.cursor_position()
    }
}

impl<Renderer> Default for Content<Renderer>
where
    Renderer: text::Renderer,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<Renderer> fmt::Debug for Content<Renderer>
where
    Renderer: text::Renderer,
    Renderer::Editor: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let internal = self.0.borrow();

        f.debug_struct("Content")
            .field("editor", &internal.editor)
            .field("is_dirty", &internal.is_dirty)
            .finish()
    }
}

struct State<Highlighter: text::Highlighter> {
    is_focused: bool,
    last_click: Option<mouse::Click>,
    drag_click: Option<mouse::click::Kind>,
    highlighter: RefCell<Highlighter>,
    highlighter_settings: Highlighter::Settings,
    highlighter_format_address: usize,
}

impl<'a, Highlighter, Message, Renderer> Widget<Message, Renderer>
    for TextEditor<'a, Highlighter, Message, Renderer>
where
    Highlighter: text::Highlighter,
    Renderer: text::Renderer,
    Renderer::Theme: StyleSheet,
{
    fn tag(&self) -> widget::tree::Tag {
        widget::tree::Tag::of::<State<Highlighter>>()
    }

    fn state(&self) -> widget::tree::State {
        widget::tree::State::new(State {
            is_focused: false,
            last_click: None,
            drag_click: None,
            highlighter: RefCell::new(Highlighter::new(
                &self.highlighter_settings,
            )),
            highlighter_settings: self.highlighter_settings.clone(),
            highlighter_format_address: self.highlighter_format as usize,
        })
    }

    fn size(&self) -> Size<Length> {
        Size {
            width: self.width,
            height: self.height,
        }
    }

    fn layout(
        &self,
        tree: &mut widget::Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> iced_renderer::core::layout::Node {
        let mut internal = self.content.0.borrow_mut();
        let state = tree.state.downcast_mut::<State<Highlighter>>();

        if state.highlighter_format_address != self.highlighter_format as usize
        {
            state.highlighter.borrow_mut().change_line(0);

            state.highlighter_format_address = self.highlighter_format as usize;
        }

        if state.highlighter_settings != self.highlighter_settings {
            state
                .highlighter
                .borrow_mut()
                .update(&self.highlighter_settings);

            state.highlighter_settings = self.highlighter_settings.clone();
        }

        internal.editor.update(
            limits.shrink(self.padding).max(),
            self.font.unwrap_or_else(|| renderer.default_font()),
            self.text_size.unwrap_or_else(|| renderer.default_size()),
            self.line_height,
            state.highlighter.borrow_mut().deref_mut(),
        );

        layout::Node::new(limits.max())
    }

    fn on_event(
        &mut self,
        tree: &mut widget::Tree,
        event: Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        _viewport: &Rectangle,
    ) -> event::Status {
        let Some(on_edit) = self.on_edit.as_ref() else {
            return event::Status::Ignored;
        };

        let state = tree.state.downcast_mut::<State<Highlighter>>();

        let Some(update) = Update::from_event(
            event,
            state,
            layout.bounds(),
            self.padding,
            cursor,
        ) else {
            return event::Status::Ignored;
        };

        match update {
            Update::Click(click) => {
                let action = match click.kind() {
                    mouse::click::Kind::Single => {
                        Action::Click(click.position())
                    }
                    mouse::click::Kind::Double => Action::SelectWord,
                    mouse::click::Kind::Triple => Action::SelectLine,
                };

                state.is_focused = true;
                state.last_click = Some(click);
                state.drag_click = Some(click.kind());

                shell.publish(on_edit(action));
            }
            Update::Unfocus => {
                state.is_focused = false;
                state.drag_click = None;
            }
            Update::Release => {
                state.drag_click = None;
            }
            Update::Action(action) => {
                shell.publish(on_edit(action));
            }
            Update::Copy => {
                if let Some(selection) = self.content.selection() {
                    clipboard.write(selection);
                }
            }
            Update::Paste => {
                if let Some(contents) = clipboard.read() {
                    shell.publish(on_edit(Action::Edit(Edit::Paste(
                        Arc::new(contents),
                    ))));
                }
            }
        }

        event::Status::Captured
    }

    fn draw(
        &self,
        tree: &widget::Tree,
        renderer: &mut Renderer,
        theme: &<Renderer as renderer::Renderer>::Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let bounds = layout.bounds();

        let mut internal = self.content.0.borrow_mut();
        let state = tree.state.downcast_ref::<State<Highlighter>>();

        internal.editor.highlight(
            self.font.unwrap_or_else(|| renderer.default_font()),
            state.highlighter.borrow_mut().deref_mut(),
            |highlight| (self.highlighter_format)(highlight, theme),
        );

        let is_disabled = self.on_edit.is_none();
        let is_mouse_over = cursor.is_over(bounds);

        let appearance = if is_disabled {
            theme.disabled(&self.style)
        } else if state.is_focused {
            theme.focused(&self.style)
        } else if is_mouse_over {
            theme.hovered(&self.style)
        } else {
            theme.active(&self.style)
        };

        renderer.fill_quad(
            renderer::Quad {
                bounds,
                border_radius: appearance.border_radius,
                border_width: appearance.border_width,
                border_color: appearance.border_color,
            },
            appearance.background,
        );

        renderer.fill_editor(
            &internal.editor,
            bounds.position()
                + Vector::new(self.padding.left, self.padding.top),
            style.text_color,
            *viewport,
        );

        let translation = Vector::new(
            bounds.x + self.padding.left,
            bounds.y + self.padding.top,
        );

        if state.is_focused {
            match internal.editor.cursor() {
                Cursor::Caret(position) => {
                    let position = position + translation;

                    if bounds.contains(position) {
                        renderer.fill_quad(
                            renderer::Quad {
                                bounds: Rectangle {
                                    x: position.x,
                                    y: position.y,
                                    width: 1.0,
                                    height: self
                                        .line_height
                                        .to_absolute(
                                            self.text_size.unwrap_or_else(
                                                || renderer.default_size(),
                                            ),
                                        )
                                        .into(),
                                },
                                border_radius: 0.0.into(),
                                border_width: 0.0,
                                border_color: Color::TRANSPARENT,
                            },
                            theme.value_color(&self.style),
                        );
                    }
                }
                Cursor::Selection(ranges) => {
                    for range in ranges.into_iter().filter_map(|range| {
                        bounds.intersection(&(range + translation))
                    }) {
                        renderer.fill_quad(
                            renderer::Quad {
                                bounds: range,
                                border_radius: 0.0.into(),
                                border_width: 0.0,
                                border_color: Color::TRANSPARENT,
                            },
                            theme.selection_color(&self.style),
                        );
                    }
                }
            }
        }
    }

    fn mouse_interaction(
        &self,
        _state: &widget::Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _viewport: &Rectangle,
        _renderer: &Renderer,
    ) -> mouse::Interaction {
        let is_disabled = self.on_edit.is_none();

        if cursor.is_over(layout.bounds()) {
            if is_disabled {
                mouse::Interaction::NotAllowed
            } else {
                mouse::Interaction::Text
            }
        } else {
            mouse::Interaction::default()
        }
    }
}

impl<'a, Highlighter, Message, Renderer>
    From<TextEditor<'a, Highlighter, Message, Renderer>>
    for Element<'a, Message, Renderer>
where
    Highlighter: text::Highlighter,
    Message: 'a,
    Renderer: text::Renderer,
    Renderer::Theme: StyleSheet,
{
    fn from(
        text_editor: TextEditor<'a, Highlighter, Message, Renderer>,
    ) -> Self {
        Self::new(text_editor)
    }
}

enum Update {
    Click(mouse::Click),
    Unfocus,
    Release,
    Action(Action),
    Copy,
    Paste,
}

impl Update {
    fn from_event<H: Highlighter>(
        event: Event,
        state: &State<H>,
        bounds: Rectangle,
        padding: Padding,
        cursor: mouse::Cursor,
    ) -> Option<Self> {
        let action = |action| Some(Update::Action(action));
        let edit = |edit| action(Action::Edit(edit));

        match event {
            Event::Mouse(event) => match event {
                mouse::Event::ButtonPressed(mouse::Button::Left) => {
                    if let Some(cursor_position) = cursor.position_in(bounds) {
                        let cursor_position = cursor_position
                            - Vector::new(padding.top, padding.left);

                        let click = mouse::Click::new(
                            cursor_position,
                            state.last_click,
                        );

                        Some(Update::Click(click))
                    } else if state.is_focused {
                        Some(Update::Unfocus)
                    } else {
                        None
                    }
                }
                mouse::Event::ButtonReleased(mouse::Button::Left) => {
                    Some(Update::Release)
                }
                mouse::Event::CursorMoved { .. } => match state.drag_click {
                    Some(mouse::click::Kind::Single) => {
                        let cursor_position = cursor.position_in(bounds)?
                            - Vector::new(padding.top, padding.left);

                        action(Action::Drag(cursor_position))
                    }
                    _ => None,
                },
                mouse::Event::WheelScrolled { delta }
                    if cursor.is_over(bounds) =>
                {
                    action(Action::Scroll {
                        lines: match delta {
                            mouse::ScrollDelta::Lines { y, .. } => {
                                if y.abs() > 0.0 {
                                    (y.signum() * -(y.abs() * 4.0).max(1.0))
                                        as i32
                                } else {
                                    0
                                }
                            }
                            mouse::ScrollDelta::Pixels { y, .. } => {
                                (-y / 4.0) as i32
                            }
                        },
                    })
                }
                _ => None,
            },
            Event::Keyboard(event) => match event {
                keyboard::Event::KeyPressed {
                    key_code,
                    modifiers,
                } if state.is_focused => {
                    if let Some(motion) = motion(key_code) {
                        let motion =
                            if platform::is_jump_modifier_pressed(modifiers) {
                                motion.widen()
                            } else {
                                motion
                            };

                        return action(if modifiers.shift() {
                            Action::Select(motion)
                        } else {
                            Action::Move(motion)
                        });
                    }

                    match key_code {
                        keyboard::KeyCode::Enter => edit(Edit::Enter),
                        keyboard::KeyCode::Backspace => edit(Edit::Backspace),
                        keyboard::KeyCode::Delete => edit(Edit::Delete),
                        keyboard::KeyCode::Escape => Some(Self::Unfocus),
                        keyboard::KeyCode::C if modifiers.command() => {
                            Some(Self::Copy)
                        }
                        keyboard::KeyCode::V
                            if modifiers.command() && !modifiers.alt() =>
                        {
                            Some(Self::Paste)
                        }
                        _ => None,
                    }
                }
                keyboard::Event::CharacterReceived(c) if state.is_focused => {
                    edit(Edit::Insert(c))
                }
                _ => None,
            },
            _ => None,
        }
    }
}

fn motion(key_code: keyboard::KeyCode) -> Option<Motion> {
    match key_code {
        keyboard::KeyCode::Left => Some(Motion::Left),
        keyboard::KeyCode::Right => Some(Motion::Right),
        keyboard::KeyCode::Up => Some(Motion::Up),
        keyboard::KeyCode::Down => Some(Motion::Down),
        keyboard::KeyCode::Home => Some(Motion::Home),
        keyboard::KeyCode::End => Some(Motion::End),
        keyboard::KeyCode::PageUp => Some(Motion::PageUp),
        keyboard::KeyCode::PageDown => Some(Motion::PageDown),
        _ => None,
    }
}

mod platform {
    use crate::core::keyboard;

    pub fn is_jump_modifier_pressed(modifiers: keyboard::Modifiers) -> bool {
        if cfg!(target_os = "macos") {
            modifiers.alt()
        } else {
            modifiers.control()
        }
    }
}
