//! Display a multi-line text input for text editing.
use crate::core::clipboard::{self, Clipboard};
use crate::core::event::{self, Event};
use crate::core::keyboard;
use crate::core::keyboard::key;
use crate::core::layout::{self, Layout};
use crate::core::mouse;
use crate::core::renderer;
use crate::core::text::editor::{Cursor, Editor as _};
use crate::core::text::highlighter::{self, Highlighter};
use crate::core::text::{self, LineHeight};
use crate::core::widget::{self, Widget};
use crate::core::{
    Background, Border, Color, Element, Length, Padding, Pixels, Rectangle,
    Shell, Size, Theme, Vector,
};

use std::cell::RefCell;
use std::fmt;
use std::ops::DerefMut;
use std::sync::Arc;

pub use text::editor::{Action, Edit, Motion};

/// A multi-line text input.
#[allow(missing_debug_implementations)]
pub struct TextEditor<
    'a,
    Highlighter,
    Message,
    Theme = crate::Theme,
    Renderer = crate::Renderer,
> where
    Highlighter: text::Highlighter,
    Theme: Catalog,
    Renderer: text::Renderer,
{
    content: &'a Content<Renderer>,
    font: Option<Renderer::Font>,
    text_size: Option<Pixels>,
    line_height: LineHeight,
    width: Length,
    height: Length,
    padding: Padding,
    class: Theme::Class<'a>,
    on_edit: Option<Box<dyn Fn(Action) -> Message + 'a>>,
    highlighter_settings: Highlighter::Settings,
    highlighter_format: fn(
        &Highlighter::Highlight,
        &Theme,
    ) -> highlighter::Format<Renderer::Font>,
}

impl<'a, Message, Theme, Renderer>
    TextEditor<'a, highlighter::PlainText, Message, Theme, Renderer>
where
    Theme: Catalog,
    Renderer: text::Renderer,
{
    /// Creates new [`TextEditor`] with the given [`Content`].
    pub fn new(content: &'a Content<Renderer>) -> Self {
        Self {
            content,
            font: None,
            text_size: None,
            line_height: LineHeight::default(),
            width: Length::Fill,
            height: Length::Shrink,
            padding: Padding::new(5.0),
            class: Theme::default(),
            on_edit: None,
            highlighter_settings: (),
            highlighter_format: |_highlight, _theme| {
                highlighter::Format::default()
            },
        }
    }
}

impl<'a, Highlighter, Message, Theme, Renderer>
    TextEditor<'a, Highlighter, Message, Theme, Renderer>
where
    Highlighter: text::Highlighter,
    Theme: Catalog,
    Renderer: text::Renderer,
{
    /// Sets the height of the [`TextEditor`].
    pub fn height(mut self, height: impl Into<Length>) -> Self {
        self.height = height.into();
        self
    }

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

    /// Sets the text size of the [`TextEditor`].
    pub fn size(mut self, size: impl Into<Pixels>) -> Self {
        self.text_size = Some(size.into());
        self
    }

    /// Sets the [`text::LineHeight`] of the [`TextEditor`].
    pub fn line_height(
        mut self,
        line_height: impl Into<text::LineHeight>,
    ) -> Self {
        self.line_height = line_height.into();
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
            &Theme,
        ) -> highlighter::Format<Renderer::Font>,
    ) -> TextEditor<'a, H, Message, Theme, Renderer> {
        TextEditor {
            content: self.content,
            font: self.font,
            text_size: self.text_size,
            line_height: self.line_height,
            width: self.width,
            height: self.height,
            padding: self.padding,
            class: self.class,
            on_edit: self.on_edit,
            highlighter_settings: settings,
            highlighter_format: to_format,
        }
    }

    /// Sets the style of the [`TextEditor`].
    #[must_use]
    pub fn style(mut self, style: impl Fn(&Theme, Status) -> Style + 'a) -> Self
    where
        Theme::Class<'a>: From<StyleFn<'a, Theme>>,
    {
        self.class = (Box::new(style) as StyleFn<'a, Theme>).into();
        self
    }

    /// Sets the style class of the [`TextEditor`].
    #[cfg(feature = "advanced")]
    #[must_use]
    pub fn class(mut self, class: impl Into<Theme::Class<'a>>) -> Self {
        self.class = class.into();
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

/// The state of a [`TextEditor`].
#[derive(Debug)]
pub struct State<Highlighter: text::Highlighter> {
    is_focused: bool,
    last_click: Option<mouse::Click>,
    drag_click: Option<mouse::click::Kind>,
    partial_scroll: f32,
    highlighter: RefCell<Highlighter>,
    highlighter_settings: Highlighter::Settings,
    highlighter_format_address: usize,
}

impl<Highlighter: text::Highlighter> State<Highlighter> {
    /// Returns whether the [`TextEditor`] is currently focused or not.
    pub fn is_focused(&self) -> bool {
        self.is_focused
    }
}

impl<'a, Highlighter, Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for TextEditor<'a, Highlighter, Message, Theme, Renderer>
where
    Highlighter: text::Highlighter,
    Theme: Catalog,
    Renderer: text::Renderer,
{
    fn tag(&self) -> widget::tree::Tag {
        widget::tree::Tag::of::<State<Highlighter>>()
    }

    fn state(&self) -> widget::tree::State {
        widget::tree::State::new(State {
            is_focused: false,
            last_click: None,
            drag_click: None,
            partial_scroll: 0.0,
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

        let limits = limits.height(self.height);

        internal.editor.update(
            limits.shrink(self.padding).max(),
            self.font.unwrap_or_else(|| renderer.default_font()),
            self.text_size.unwrap_or_else(|| renderer.default_size()),
            self.line_height,
            state.highlighter.borrow_mut().deref_mut(),
        );

        match self.height {
            Length::Fill | Length::FillPortion(_) | Length::Fixed(_) => {
                layout::Node::new(limits.max())
            }
            Length::Shrink => {
                let min_bounds = internal.editor.min_bounds();

                layout::Node::new(
                    limits
                        .height(min_bounds.height)
                        .max()
                        .expand(Size::new(0.0, self.padding.vertical())),
                )
            }
        }
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
            Update::Scroll(lines) => {
                let bounds = self.content.0.borrow().editor.bounds();

                if bounds.height >= i32::MAX as f32 {
                    return event::Status::Ignored;
                }

                let lines = lines + state.partial_scroll;
                state.partial_scroll = lines.fract();

                shell.publish(on_edit(Action::Scroll {
                    lines: lines as i32,
                }));
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
                    clipboard.write(clipboard::Kind::Standard, selection);
                }
            }
            Update::Cut => {
                if let Some(selection) = self.content.selection() {
                    clipboard.write(clipboard::Kind::Standard, selection);
                    shell.publish(on_edit(Action::Edit(Edit::Delete)));
                }
            }
            Update::Paste => {
                if let Some(contents) =
                    clipboard.read(clipboard::Kind::Standard)
                {
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
        theme: &Theme,
        defaults: &renderer::Style,
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

        let status = if is_disabled {
            Status::Disabled
        } else if state.is_focused {
            Status::Focused
        } else if is_mouse_over {
            Status::Hovered
        } else {
            Status::Active
        };

        let style = theme.style(&self.class, status);

        renderer.fill_quad(
            renderer::Quad {
                bounds,
                border: style.border,
                ..renderer::Quad::default()
            },
            style.background,
        );

        renderer.fill_editor(
            &internal.editor,
            bounds.position()
                + Vector::new(self.padding.left, self.padding.top),
            defaults.text_color,
            *viewport,
        );

        let translation = Vector::new(
            bounds.x + self.padding.left,
            bounds.y + self.padding.top,
        );

        if state.is_focused {
            match internal.editor.cursor() {
                Cursor::Caret(position) => {
                    let cursor =
                        Rectangle::new(
                            position + translation,
                            Size::new(
                                1.0,
                                self.line_height
                                    .to_absolute(self.text_size.unwrap_or_else(
                                        || renderer.default_size(),
                                    ))
                                    .into(),
                            ),
                        );

                    if let Some(clipped_cursor) = bounds.intersection(&cursor) {
                        renderer.fill_quad(
                            renderer::Quad {
                                bounds: Rectangle {
                                    x: clipped_cursor.x.floor(),
                                    y: clipped_cursor.y,
                                    width: clipped_cursor.width,
                                    height: clipped_cursor.height,
                                },
                                ..renderer::Quad::default()
                            },
                            style.value,
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
                                ..renderer::Quad::default()
                            },
                            style.selection,
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

impl<'a, Highlighter, Message, Theme, Renderer>
    From<TextEditor<'a, Highlighter, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Highlighter: text::Highlighter,
    Message: 'a,
    Theme: Catalog + 'a,
    Renderer: text::Renderer,
{
    fn from(
        text_editor: TextEditor<'a, Highlighter, Message, Theme, Renderer>,
    ) -> Self {
        Self::new(text_editor)
    }
}

enum Update {
    Click(mouse::Click),
    Scroll(f32),
    Unfocus,
    Release,
    Action(Action),
    Copy,
    Cut,
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
                    Some(Update::Scroll(match delta {
                        mouse::ScrollDelta::Lines { y, .. } => {
                            if y.abs() > 0.0 {
                                y.signum() * -(y.abs() * 4.0).max(1.0)
                            } else {
                                0.0
                            }
                        }
                        mouse::ScrollDelta::Pixels { y, .. } => -y / 4.0,
                    }))
                }
                _ => None,
            },
            Event::Keyboard(event) => match event {
                keyboard::Event::KeyPressed {
                    key,
                    modifiers,
                    text,
                    ..
                } if state.is_focused => {
                    match key.as_ref() {
                        keyboard::Key::Named(key::Named::Enter) => {
                            return edit(Edit::Enter);
                        }
                        keyboard::Key::Named(key::Named::Backspace) => {
                            return edit(Edit::Backspace);
                        }
                        keyboard::Key::Named(key::Named::Delete) => {
                            return edit(Edit::Delete);
                        }
                        keyboard::Key::Named(key::Named::Escape) => {
                            return Some(Self::Unfocus);
                        }
                        keyboard::Key::Character("c")
                            if modifiers.command() =>
                        {
                            return Some(Self::Copy);
                        }
                        keyboard::Key::Character("x")
                            if modifiers.command() =>
                        {
                            return Some(Self::Cut);
                        }
                        keyboard::Key::Character("v")
                            if modifiers.command() && !modifiers.alt() =>
                        {
                            return Some(Self::Paste);
                        }
                        keyboard::Key::Character("a")
                            if modifiers.command() =>
                        {
                            return Some(Self::Action(Action::SelectAll));
                        }
                        _ => {}
                    }

                    if let Some(text) = text {
                        if let Some(c) = text.chars().find(|c| !c.is_control())
                        {
                            return edit(Edit::Insert(c));
                        }
                    }

                    if let keyboard::Key::Named(named_key) = key.as_ref() {
                        if let Some(motion) = motion(named_key) {
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

                            return action(if modifiers.shift() {
                                Action::Select(motion)
                            } else {
                                Action::Move(motion)
                            });
                        }
                    }

                    None
                }
                _ => None,
            },
            _ => None,
        }
    }
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

/// The possible status of a [`TextEditor`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    /// The [`TextEditor`] can be interacted with.
    Active,
    /// The [`TextEditor`] is being hovered.
    Hovered,
    /// The [`TextEditor`] is focused.
    Focused,
    /// The [`TextEditor`] cannot be interacted with.
    Disabled,
}

/// The appearance of a text input.
#[derive(Debug, Clone, Copy)]
pub struct Style {
    /// The [`Background`] of the text input.
    pub background: Background,
    /// The [`Border`] of the text input.
    pub border: Border,
    /// The [`Color`] of the icon of the text input.
    pub icon: Color,
    /// The [`Color`] of the placeholder of the text input.
    pub placeholder: Color,
    /// The [`Color`] of the value of the text input.
    pub value: Color,
    /// The [`Color`] of the selection of the text input.
    pub selection: Color,
}

/// The theme catalog of a [`TextEditor`].
pub trait Catalog {
    /// The item class of the [`Catalog`].
    type Class<'a>;

    /// The default class produced by the [`Catalog`].
    fn default<'a>() -> Self::Class<'a>;

    /// The [`Style`] of a class with the given status.
    fn style(&self, class: &Self::Class<'_>, status: Status) -> Style;
}

/// A styling function for a [`TextEditor`].
pub type StyleFn<'a, Theme> = Box<dyn Fn(&Theme, Status) -> Style + 'a>;

impl Catalog for Theme {
    type Class<'a> = StyleFn<'a, Self>;

    fn default<'a>() -> Self::Class<'a> {
        Box::new(default)
    }

    fn style(&self, class: &Self::Class<'_>, status: Status) -> Style {
        class(self, status)
    }
}

/// The default style of a [`TextEditor`].
pub fn default(theme: &Theme, status: Status) -> Style {
    let palette = theme.extended_palette();

    let active = Style {
        background: Background::Color(palette.background.base.color),
        border: Border {
            radius: 2.0.into(),
            width: 1.0,
            color: palette.background.strong.color,
        },
        icon: palette.background.weak.text,
        placeholder: palette.background.strong.color,
        value: palette.background.base.text,
        selection: palette.primary.weak.color,
    };

    match status {
        Status::Active => active,
        Status::Hovered => Style {
            border: Border {
                color: palette.background.base.text,
                ..active.border
            },
            ..active
        },
        Status::Focused => Style {
            border: Border {
                color: palette.primary.strong.color,
                ..active.border
            },
            ..active
        },
        Status::Disabled => Style {
            background: Background::Color(palette.background.weak.color),
            value: active.placeholder,
            ..active
        },
    }
}
