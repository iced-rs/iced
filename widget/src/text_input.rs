//! Text inputs display fields that can be filled with text.
//!
//! # Example
//! ```no_run
//! # mod iced { pub mod widget { pub use iced_widget::*; } pub use iced_widget::Renderer; pub use iced_widget::core::*; }
//! # pub type Element<'a, Message> = iced_widget::core::Element<'a, Message, iced_widget::Theme, iced_widget::Renderer>;
//! #
//! use iced::widget::text_input;
//!
//! struct State {
//!    content: String,
//! }
//!
//! #[derive(Debug, Clone)]
//! enum Message {
//!     ContentChanged(String)
//! }
//!
//! fn view(state: &State) -> Element<'_, Message> {
//!     text_input("Type something here...", &state.content)
//!         .on_input(Message::ContentChanged)
//!         .into()
//! }
//!
//! fn update(state: &mut State, message: Message) {
//!     match message {
//!         Message::ContentChanged(content) => {
//!             state.content = content;
//!         }
//!     }
//! }
//! ```
mod editor;
mod value;

pub mod cursor;

pub use cursor::Cursor;
pub use value::Value;

use editor::Editor;

use crate::core::alignment;
use crate::core::clipboard::{self, Clipboard};
use crate::core::input_method;
use crate::core::keyboard;
use crate::core::keyboard::key;
use crate::core::layout;
use crate::core::mouse::{self, click};
use crate::core::renderer;
use crate::core::text::paragraph::{self, Paragraph as _};
use crate::core::text::{self, Text};
use crate::core::time::{Duration, Instant};
use crate::core::touch;
use crate::core::widget;
use crate::core::widget::operation::{self, Operation};
use crate::core::widget::tree::{self, Tree};
use crate::core::window;
use crate::core::{
    Alignment, Background, Border, Color, Element, Event, InputMethod, Layout,
    Length, Padding, Pixels, Point, Rectangle, Shell, Size, Theme, Vector,
    Widget,
};
use crate::runtime::Action;
use crate::runtime::task::{self, Task};

/// A field that can be filled with text.
///
/// # Example
/// ```no_run
/// # mod iced { pub mod widget { pub use iced_widget::*; } pub use iced_widget::Renderer; pub use iced_widget::core::*; }
/// # pub type Element<'a, Message> = iced_widget::core::Element<'a, Message, iced_widget::Theme, iced_widget::Renderer>;
/// #
/// use iced::widget::text_input;
///
/// struct State {
///    content: String,
/// }
///
/// #[derive(Debug, Clone)]
/// enum Message {
///     ContentChanged(String)
/// }
///
/// fn view(state: &State) -> Element<'_, Message> {
///     text_input("Type something here...", &state.content)
///         .on_input(Message::ContentChanged)
///         .into()
/// }
///
/// fn update(state: &mut State, message: Message) {
///     match message {
///         Message::ContentChanged(content) => {
///             state.content = content;
///         }
///     }
/// }
/// ```
#[allow(missing_debug_implementations)]
pub struct TextInput<
    'a,
    Message,
    Theme = crate::Theme,
    Renderer = crate::Renderer,
> where
    Theme: Catalog,
    Renderer: text::Renderer,
{
    id: Option<Id>,
    placeholder: String,
    value: Value,
    is_secure: bool,
    font: Option<Renderer::Font>,
    width: Length,
    padding: Padding,
    size: Option<Pixels>,
    line_height: text::LineHeight,
    alignment: alignment::Horizontal,
    on_input: Option<Box<dyn Fn(String) -> Message + 'a>>,
    on_paste: Option<Box<dyn Fn(String) -> Message + 'a>>,
    on_submit: Option<Message>,
    icon: Option<Icon<Renderer::Font>>,
    class: Theme::Class<'a>,
    last_status: Option<Status>,
}

/// The default [`Padding`] of a [`TextInput`].
pub const DEFAULT_PADDING: Padding = Padding::new(5.0);

impl<'a, Message, Theme, Renderer> TextInput<'a, Message, Theme, Renderer>
where
    Message: Clone,
    Theme: Catalog,
    Renderer: text::Renderer,
{
    /// Creates a new [`TextInput`] with the given placeholder and
    /// its current value.
    pub fn new(placeholder: &str, value: &str) -> Self {
        TextInput {
            id: None,
            placeholder: String::from(placeholder),
            value: Value::new(value),
            is_secure: false,
            font: None,
            width: Length::Fill,
            padding: DEFAULT_PADDING,
            size: None,
            line_height: text::LineHeight::default(),
            alignment: alignment::Horizontal::Left,
            on_input: None,
            on_paste: None,
            on_submit: None,
            icon: None,
            class: Theme::default(),
            last_status: None,
        }
    }

    /// Sets the [`Id`] of the [`TextInput`].
    pub fn id(mut self, id: impl Into<Id>) -> Self {
        self.id = Some(id.into());
        self
    }

    /// Converts the [`TextInput`] into a secure password input.
    pub fn secure(mut self, is_secure: bool) -> Self {
        self.is_secure = is_secure;
        self
    }

    /// Sets the message that should be produced when some text is typed into
    /// the [`TextInput`].
    ///
    /// If this method is not called, the [`TextInput`] will be disabled.
    pub fn on_input(
        mut self,
        on_input: impl Fn(String) -> Message + 'a,
    ) -> Self {
        self.on_input = Some(Box::new(on_input));
        self
    }

    /// Sets the message that should be produced when some text is typed into
    /// the [`TextInput`], if `Some`.
    ///
    /// If `None`, the [`TextInput`] will be disabled.
    pub fn on_input_maybe(
        mut self,
        on_input: Option<impl Fn(String) -> Message + 'a>,
    ) -> Self {
        self.on_input = on_input.map(|f| Box::new(f) as _);
        self
    }

    /// Sets the message that should be produced when the [`TextInput`] is
    /// focused and the enter key is pressed.
    pub fn on_submit(mut self, message: Message) -> Self {
        self.on_submit = Some(message);
        self
    }

    /// Sets the message that should be produced when the [`TextInput`] is
    /// focused and the enter key is pressed, if `Some`.
    pub fn on_submit_maybe(mut self, on_submit: Option<Message>) -> Self {
        self.on_submit = on_submit;
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

    /// Sets the message that should be produced when some text is pasted into
    /// the [`TextInput`], if `Some`.
    pub fn on_paste_maybe(
        mut self,
        on_paste: Option<impl Fn(String) -> Message + 'a>,
    ) -> Self {
        self.on_paste = on_paste.map(|f| Box::new(f) as _);
        self
    }

    /// Sets the [`Font`] of the [`TextInput`].
    ///
    /// [`Font`]: text::Renderer::Font
    pub fn font(mut self, font: Renderer::Font) -> Self {
        self.font = Some(font);
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
        self.size = Some(size.into());
        self
    }

    /// Sets the [`text::LineHeight`] of the [`TextInput`].
    pub fn line_height(
        mut self,
        line_height: impl Into<text::LineHeight>,
    ) -> Self {
        self.line_height = line_height.into();
        self
    }

    /// Sets the horizontal alignment of the [`TextInput`].
    pub fn align_x(
        mut self,
        alignment: impl Into<alignment::Horizontal>,
    ) -> Self {
        self.alignment = alignment.into();
        self
    }

    /// Sets the style of the [`TextInput`].
    #[must_use]
    pub fn style(mut self, style: impl Fn(&Theme, Status) -> Style + 'a) -> Self
    where
        Theme::Class<'a>: From<StyleFn<'a, Theme>>,
    {
        self.class = (Box::new(style) as StyleFn<'a, Theme>).into();
        self
    }

    /// Sets the style class of the [`TextInput`].
    #[must_use]
    pub fn class(mut self, class: impl Into<Theme::Class<'a>>) -> Self {
        self.class = class.into();
        self
    }

    /// Lays out the [`TextInput`], overriding its [`Value`] if provided.
    ///
    /// [`Renderer`]: text::Renderer
    pub fn layout(
        &self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
        value: Option<&Value>,
    ) -> layout::Node {
        let state = tree.state.downcast_mut::<State<Renderer::Paragraph>>();
        let value = value.unwrap_or(&self.value);

        let font = self.font.unwrap_or_else(|| renderer.default_font());
        let text_size = self.size.unwrap_or_else(|| renderer.default_size());
        let padding = self.padding.fit(Size::ZERO, limits.max());
        let height = self.line_height.to_absolute(text_size);

        let limits = limits.width(self.width).shrink(padding);
        let text_bounds = limits.resolve(self.width, height, Size::ZERO);

        let placeholder_text = Text {
            font,
            line_height: self.line_height,
            content: self.placeholder.as_str(),
            bounds: Size::new(f32::INFINITY, text_bounds.height),
            size: text_size,
            align_x: text::Alignment::Default,
            align_y: alignment::Vertical::Center,
            shaping: text::Shaping::Advanced,
            wrapping: text::Wrapping::default(),
        };

        let _ = state.placeholder.update(placeholder_text);

        let secure_value = self.is_secure.then(|| value.secure());
        let value = secure_value.as_ref().unwrap_or(value);

        let _ = state.value.update(Text {
            content: &value.to_string(),
            ..placeholder_text
        });

        if let Some(icon) = &self.icon {
            let mut content = [0; 4];

            let icon_text = Text {
                line_height: self.line_height,
                content: icon.code_point.encode_utf8(&mut content) as &_,
                font: icon.font,
                size: icon.size.unwrap_or_else(|| renderer.default_size()),
                bounds: Size::new(f32::INFINITY, text_bounds.height),
                align_x: text::Alignment::Center,
                align_y: alignment::Vertical::Center,
                shaping: text::Shaping::Advanced,
                wrapping: text::Wrapping::default(),
            };

            let _ = state.icon.update(icon_text);

            let icon_width = state.icon.min_width();

            let (text_position, icon_position) = match icon.side {
                Side::Left => (
                    Point::new(
                        padding.left + icon_width + icon.spacing,
                        padding.top,
                    ),
                    Point::new(padding.left, padding.top),
                ),
                Side::Right => (
                    Point::new(padding.left, padding.top),
                    Point::new(
                        padding.left + text_bounds.width - icon_width,
                        padding.top,
                    ),
                ),
            };

            let text_node = layout::Node::new(
                text_bounds - Size::new(icon_width + icon.spacing, 0.0),
            )
            .move_to(text_position);

            let icon_node =
                layout::Node::new(Size::new(icon_width, text_bounds.height))
                    .move_to(icon_position);

            layout::Node::with_children(
                text_bounds.expand(padding),
                vec![text_node, icon_node],
            )
        } else {
            let text = layout::Node::new(text_bounds)
                .move_to(Point::new(padding.left, padding.top));

            layout::Node::with_children(text_bounds.expand(padding), vec![text])
        }
    }

    fn input_method<'b>(
        &self,
        state: &'b State<Renderer::Paragraph>,
        layout: Layout<'_>,
        value: &Value,
    ) -> InputMethod<&'b str> {
        let Some(Focus {
            is_window_focused: true,
            ..
        }) = &state.is_focused
        else {
            return InputMethod::Disabled;
        };

        let secure_value = self.is_secure.then(|| value.secure());
        let value = secure_value.as_ref().unwrap_or(value);

        let text_bounds = layout.children().next().unwrap().bounds();

        let caret_index = match state.cursor.state(value) {
            cursor::State::Index(position) => position,
            cursor::State::Selection { start, end } => start.min(end),
        };

        let text = state.value.raw();
        let (cursor_x, scroll_offset) =
            measure_cursor_and_scroll_offset(text, text_bounds, caret_index);

        let alignment_offset = alignment_offset(
            text_bounds.width,
            text.min_width(),
            self.alignment,
        );

        let x = (text_bounds.x + cursor_x).floor() - scroll_offset
            + alignment_offset;

        InputMethod::Enabled {
            position: Point::new(x, text_bounds.y + text_bounds.height),
            purpose: if self.is_secure {
                input_method::Purpose::Secure
            } else {
                input_method::Purpose::Normal
            },
            preedit: state.preedit.as_ref().map(input_method::Preedit::as_ref),
        }
    }

    /// Draws the [`TextInput`] with the given [`Renderer`], overriding its
    /// [`Value`] if provided.
    ///
    /// [`Renderer`]: text::Renderer
    pub fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        layout: Layout<'_>,
        _cursor: mouse::Cursor,
        value: Option<&Value>,
        viewport: &Rectangle,
    ) {
        let state = tree.state.downcast_ref::<State<Renderer::Paragraph>>();
        let value = value.unwrap_or(&self.value);
        let is_disabled = self.on_input.is_none();

        let secure_value = self.is_secure.then(|| value.secure());
        let value = secure_value.as_ref().unwrap_or(value);

        let bounds = layout.bounds();

        let mut children_layout = layout.children();
        let text_bounds = children_layout.next().unwrap().bounds();

        let style = theme
            .style(&self.class, self.last_status.unwrap_or(Status::Disabled));

        renderer.fill_quad(
            renderer::Quad {
                bounds,
                border: style.border,
                ..renderer::Quad::default()
            },
            style.background,
        );

        if self.icon.is_some() {
            let icon_layout = children_layout.next().unwrap();

            let icon = state.icon.raw();

            renderer.fill_paragraph(
                icon,
                icon_layout.bounds().anchor(
                    icon.min_bounds(),
                    Alignment::Center,
                    Alignment::Center,
                ),
                style.icon,
                *viewport,
            );
        }

        let text = value.to_string();

        let (cursor, offset, is_selecting) = if let Some(focus) = state
            .is_focused
            .as_ref()
            .filter(|focus| focus.is_window_focused)
        {
            match state.cursor.state(value) {
                cursor::State::Index(position) => {
                    let (text_value_width, offset) =
                        measure_cursor_and_scroll_offset(
                            state.value.raw(),
                            text_bounds,
                            position,
                        );

                    let is_cursor_visible = !is_disabled
                        && ((focus.now - focus.updated_at).as_millis()
                            / CURSOR_BLINK_INTERVAL_MILLIS)
                            % 2
                            == 0;

                    let cursor = if is_cursor_visible {
                        Some((
                            renderer::Quad {
                                bounds: Rectangle {
                                    x: (text_bounds.x + text_value_width)
                                        .floor(),
                                    y: text_bounds.y,
                                    width: 1.0,
                                    height: text_bounds.height,
                                },
                                ..renderer::Quad::default()
                            },
                            style.value,
                        ))
                    } else {
                        None
                    };

                    (cursor, offset, false)
                }
                cursor::State::Selection { start, end } => {
                    let left = start.min(end);
                    let right = end.max(start);

                    let (left_position, left_offset) =
                        measure_cursor_and_scroll_offset(
                            state.value.raw(),
                            text_bounds,
                            left,
                        );

                    let (right_position, right_offset) =
                        measure_cursor_and_scroll_offset(
                            state.value.raw(),
                            text_bounds,
                            right,
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
                                ..renderer::Quad::default()
                            },
                            style.selection,
                        )),
                        if end == right {
                            right_offset
                        } else {
                            left_offset
                        },
                        true,
                    )
                }
            }
        } else {
            (None, 0.0, false)
        };

        let draw = |renderer: &mut Renderer, viewport| {
            let paragraph = if text.is_empty()
                && state
                    .preedit
                    .as_ref()
                    .map(|preedit| preedit.content.is_empty())
                    .unwrap_or(true)
            {
                state.placeholder.raw()
            } else {
                state.value.raw()
            };

            let alignment_offset = alignment_offset(
                text_bounds.width,
                paragraph.min_width(),
                self.alignment,
            );

            if let Some((cursor, color)) = cursor {
                renderer.with_translation(
                    Vector::new(alignment_offset - offset, 0.0),
                    |renderer| {
                        renderer.fill_quad(cursor, color);
                    },
                );
            } else {
                renderer.with_translation(Vector::ZERO, |_| {});
            }

            renderer.fill_paragraph(
                paragraph,
                text_bounds.anchor(
                    paragraph.min_bounds(),
                    Alignment::Start,
                    Alignment::Center,
                ) + Vector::new(alignment_offset - offset, 0.0),
                if text.is_empty() {
                    style.placeholder
                } else {
                    style.value
                },
                viewport,
            );
        };

        if is_selecting {
            renderer
                .with_layer(text_bounds, |renderer| draw(renderer, *viewport));
        } else {
            draw(renderer, text_bounds);
        }
    }
}

impl<Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for TextInput<'_, Message, Theme, Renderer>
where
    Message: Clone,
    Theme: Catalog,
    Renderer: text::Renderer,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State<Renderer::Paragraph>>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State::<Renderer::Paragraph>::new())
    }

    fn diff(&self, tree: &mut Tree) {
        let state = tree.state.downcast_mut::<State<Renderer::Paragraph>>();

        // Stop pasting if input becomes disabled
        if self.on_input.is_none() {
            state.is_pasting = None;
        }
    }

    fn size(&self) -> Size<Length> {
        Size {
            width: self.width,
            height: Length::Shrink,
        }
    }

    fn layout(
        &self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        self.layout(tree, renderer, limits, None)
    }

    fn operate(
        &self,
        tree: &mut Tree,
        layout: Layout<'_>,
        _renderer: &Renderer,
        operation: &mut dyn Operation,
    ) {
        let state = tree.state.downcast_mut::<State<Renderer::Paragraph>>();

        operation.focusable(
            self.id.as_ref().map(|id| &id.0),
            layout.bounds(),
            state,
        );

        operation.text_input(
            self.id.as_ref().map(|id| &id.0),
            layout.bounds(),
            state,
        );
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        _viewport: &Rectangle,
    ) {
        let update_cache = |state, value| {
            replace_paragraph(
                renderer,
                state,
                layout,
                value,
                self.font,
                self.size,
                self.line_height,
            );
        };

        match &event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
            | Event::Touch(touch::Event::FingerPressed { .. }) => {
                let state = state::<Renderer>(tree);
                let cursor_before = state.cursor;

                let click_position = cursor.position_over(layout.bounds());

                state.is_focused = if click_position.is_some() {
                    let now = Instant::now();

                    Some(Focus {
                        updated_at: now,
                        now,
                        is_window_focused: true,
                    })
                } else {
                    None
                };

                if let Some(cursor_position) = click_position {
                    let text_layout = layout.children().next().unwrap();

                    let target = {
                        let text_bounds = text_layout.bounds();

                        let alignment_offset = alignment_offset(
                            text_bounds.width,
                            state.value.raw().min_width(),
                            self.alignment,
                        );

                        cursor_position.x - text_bounds.x - alignment_offset
                    };

                    let click = mouse::Click::new(
                        cursor_position,
                        mouse::Button::Left,
                        state.last_click,
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
                                    text_layout.bounds(),
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
                                    state.cursor.start(&self.value),
                                    position,
                                );
                            } else {
                                state.cursor.move_to(position);
                            }
                            state.is_dragging = true;
                        }
                        click::Kind::Double => {
                            if self.is_secure {
                                state.cursor.select_all(&self.value);
                            } else {
                                let position = find_cursor_position(
                                    text_layout.bounds(),
                                    &self.value,
                                    state,
                                    target,
                                )
                                .unwrap_or(0);

                                state.cursor.select_range(
                                    self.value.previous_start_of_word(position),
                                    self.value.next_end_of_word(position),
                                );
                            }

                            state.is_dragging = false;
                        }
                        click::Kind::Triple => {
                            state.cursor.select_all(&self.value);
                            state.is_dragging = false;
                        }
                    }

                    state.last_click = Some(click);

                    if cursor_before != state.cursor {
                        shell.request_redraw();
                    }

                    shell.capture_event();
                }
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left))
            | Event::Touch(touch::Event::FingerLifted { .. })
            | Event::Touch(touch::Event::FingerLost { .. }) => {
                state::<Renderer>(tree).is_dragging = false;
            }
            Event::Mouse(mouse::Event::CursorMoved { position })
            | Event::Touch(touch::Event::FingerMoved { position, .. }) => {
                let state = state::<Renderer>(tree);

                if state.is_dragging {
                    let text_layout = layout.children().next().unwrap();

                    let target = {
                        let text_bounds = text_layout.bounds();

                        let alignment_offset = alignment_offset(
                            text_bounds.width,
                            state.value.raw().min_width(),
                            self.alignment,
                        );

                        position.x - text_bounds.x - alignment_offset
                    };

                    let value = if self.is_secure {
                        self.value.secure()
                    } else {
                        self.value.clone()
                    };

                    let position = find_cursor_position(
                        text_layout.bounds(),
                        &value,
                        state,
                        target,
                    )
                    .unwrap_or(0);

                    let selection_before = state.cursor.selection(&value);

                    state
                        .cursor
                        .select_range(state.cursor.start(&value), position);

                    if let Some(focus) = &mut state.is_focused {
                        focus.updated_at = Instant::now();
                    }

                    if selection_before != state.cursor.selection(&value) {
                        shell.request_redraw();
                    }

                    shell.capture_event();
                }
            }
            Event::Keyboard(keyboard::Event::KeyPressed {
                key, text, ..
            }) => {
                let state = state::<Renderer>(tree);

                if let Some(focus) = &mut state.is_focused {
                    let modifiers = state.keyboard_modifiers;

                    match key.as_ref() {
                        keyboard::Key::Character("c")
                            if state.keyboard_modifiers.command()
                                && !self.is_secure =>
                        {
                            if let Some((start, end)) =
                                state.cursor.selection(&self.value)
                            {
                                clipboard.write(
                                    clipboard::Kind::Standard,
                                    self.value.select(start, end).to_string(),
                                );
                            }

                            shell.capture_event();
                            return;
                        }
                        keyboard::Key::Character("x")
                            if state.keyboard_modifiers.command()
                                && !self.is_secure =>
                        {
                            let Some(on_input) = &self.on_input else {
                                return;
                            };

                            if let Some((start, end)) =
                                state.cursor.selection(&self.value)
                            {
                                clipboard.write(
                                    clipboard::Kind::Standard,
                                    self.value.select(start, end).to_string(),
                                );
                            }

                            let mut editor =
                                Editor::new(&mut self.value, &mut state.cursor);
                            editor.delete();

                            let message = (on_input)(editor.contents());
                            shell.publish(message);
                            shell.capture_event();

                            focus.updated_at = Instant::now();
                            update_cache(state, &self.value);
                            return;
                        }
                        keyboard::Key::Character("v")
                            if state.keyboard_modifiers.command()
                                && !state.keyboard_modifiers.alt() =>
                        {
                            let Some(on_input) = &self.on_input else {
                                return;
                            };

                            let content = match state.is_pasting.take() {
                                Some(content) => content,
                                None => {
                                    let content: String = clipboard
                                        .read(clipboard::Kind::Standard)
                                        .unwrap_or_default()
                                        .chars()
                                        .filter(|c| !c.is_control())
                                        .collect();

                                    Value::new(&content)
                                }
                            };

                            let mut editor =
                                Editor::new(&mut self.value, &mut state.cursor);
                            editor.paste(content.clone());

                            let message = if let Some(paste) = &self.on_paste {
                                (paste)(editor.contents())
                            } else {
                                (on_input)(editor.contents())
                            };
                            shell.publish(message);
                            shell.capture_event();

                            state.is_pasting = Some(content);
                            focus.updated_at = Instant::now();
                            update_cache(state, &self.value);
                            return;
                        }
                        keyboard::Key::Character("a")
                            if state.keyboard_modifiers.command() =>
                        {
                            let cursor_before = state.cursor;

                            state.cursor.select_all(&self.value);

                            if cursor_before != state.cursor {
                                focus.updated_at = Instant::now();

                                shell.request_redraw();
                            }

                            shell.capture_event();
                            return;
                        }
                        _ => {}
                    }

                    if let Some(text) = text {
                        let Some(on_input) = &self.on_input else {
                            return;
                        };

                        state.is_pasting = None;

                        if let Some(c) =
                            text.chars().next().filter(|c| !c.is_control())
                        {
                            let mut editor =
                                Editor::new(&mut self.value, &mut state.cursor);

                            editor.insert(c);

                            let message = (on_input)(editor.contents());
                            shell.publish(message);
                            shell.capture_event();

                            focus.updated_at = Instant::now();
                            update_cache(state, &self.value);
                            return;
                        }
                    }

                    match key.as_ref() {
                        keyboard::Key::Named(key::Named::Enter) => {
                            if let Some(on_submit) = self.on_submit.clone() {
                                shell.publish(on_submit);
                                shell.capture_event();
                            }
                        }
                        keyboard::Key::Named(key::Named::Backspace) => {
                            let Some(on_input) = &self.on_input else {
                                return;
                            };

                            if modifiers.jump()
                                && state.cursor.selection(&self.value).is_none()
                            {
                                if self.is_secure {
                                    let cursor_pos =
                                        state.cursor.end(&self.value);
                                    state.cursor.select_range(0, cursor_pos);
                                } else {
                                    state
                                        .cursor
                                        .select_left_by_words(&self.value);
                                }
                            }

                            let mut editor =
                                Editor::new(&mut self.value, &mut state.cursor);
                            editor.backspace();

                            let message = (on_input)(editor.contents());
                            shell.publish(message);
                            shell.capture_event();

                            focus.updated_at = Instant::now();
                            update_cache(state, &self.value);
                        }
                        keyboard::Key::Named(key::Named::Delete) => {
                            let Some(on_input) = &self.on_input else {
                                return;
                            };

                            if modifiers.jump()
                                && state.cursor.selection(&self.value).is_none()
                            {
                                if self.is_secure {
                                    let cursor_pos =
                                        state.cursor.end(&self.value);
                                    state.cursor.select_range(
                                        cursor_pos,
                                        self.value.len(),
                                    );
                                } else {
                                    state
                                        .cursor
                                        .select_right_by_words(&self.value);
                                }
                            }

                            let mut editor =
                                Editor::new(&mut self.value, &mut state.cursor);
                            editor.delete();

                            let message = (on_input)(editor.contents());
                            shell.publish(message);
                            shell.capture_event();

                            focus.updated_at = Instant::now();
                            update_cache(state, &self.value);
                        }
                        keyboard::Key::Named(key::Named::Home) => {
                            let cursor_before = state.cursor;

                            if modifiers.shift() {
                                state.cursor.select_range(
                                    state.cursor.start(&self.value),
                                    0,
                                );
                            } else {
                                state.cursor.move_to(0);
                            }

                            if cursor_before != state.cursor {
                                focus.updated_at = Instant::now();

                                shell.request_redraw();
                            }

                            shell.capture_event();
                        }
                        keyboard::Key::Named(key::Named::End) => {
                            let cursor_before = state.cursor;

                            if modifiers.shift() {
                                state.cursor.select_range(
                                    state.cursor.start(&self.value),
                                    self.value.len(),
                                );
                            } else {
                                state.cursor.move_to(self.value.len());
                            }

                            if cursor_before != state.cursor {
                                focus.updated_at = Instant::now();

                                shell.request_redraw();
                            }

                            shell.capture_event();
                        }
                        keyboard::Key::Named(key::Named::ArrowLeft)
                            if modifiers.macos_command() =>
                        {
                            let cursor_before = state.cursor;

                            if modifiers.shift() {
                                state.cursor.select_range(
                                    state.cursor.start(&self.value),
                                    0,
                                );
                            } else {
                                state.cursor.move_to(0);
                            }

                            if cursor_before != state.cursor {
                                focus.updated_at = Instant::now();

                                shell.request_redraw();
                            }

                            shell.capture_event();
                        }
                        keyboard::Key::Named(key::Named::ArrowRight)
                            if modifiers.macos_command() =>
                        {
                            let cursor_before = state.cursor;

                            if modifiers.shift() {
                                state.cursor.select_range(
                                    state.cursor.start(&self.value),
                                    self.value.len(),
                                );
                            } else {
                                state.cursor.move_to(self.value.len());
                            }

                            if cursor_before != state.cursor {
                                focus.updated_at = Instant::now();

                                shell.request_redraw();
                            }

                            shell.capture_event();
                        }
                        keyboard::Key::Named(key::Named::ArrowLeft) => {
                            let cursor_before = state.cursor;

                            if modifiers.jump() && !self.is_secure {
                                if modifiers.shift() {
                                    state
                                        .cursor
                                        .select_left_by_words(&self.value);
                                } else {
                                    state
                                        .cursor
                                        .move_left_by_words(&self.value);
                                }
                            } else if modifiers.shift() {
                                state.cursor.select_left(&self.value);
                            } else {
                                state.cursor.move_left(&self.value);
                            }

                            if cursor_before != state.cursor {
                                focus.updated_at = Instant::now();

                                shell.request_redraw();
                            }

                            shell.capture_event();
                        }
                        keyboard::Key::Named(key::Named::ArrowRight) => {
                            let cursor_before = state.cursor;

                            if modifiers.jump() && !self.is_secure {
                                if modifiers.shift() {
                                    state
                                        .cursor
                                        .select_right_by_words(&self.value);
                                } else {
                                    state
                                        .cursor
                                        .move_right_by_words(&self.value);
                                }
                            } else if modifiers.shift() {
                                state.cursor.select_right(&self.value);
                            } else {
                                state.cursor.move_right(&self.value);
                            }

                            if cursor_before != state.cursor {
                                focus.updated_at = Instant::now();

                                shell.request_redraw();
                            }

                            shell.capture_event();
                        }
                        keyboard::Key::Named(key::Named::Escape) => {
                            state.is_focused = None;
                            state.is_dragging = false;
                            state.is_pasting = None;

                            state.keyboard_modifiers =
                                keyboard::Modifiers::default();

                            shell.capture_event();
                        }
                        _ => {}
                    }
                }
            }
            Event::Keyboard(keyboard::Event::KeyReleased { key, .. }) => {
                let state = state::<Renderer>(tree);

                if state.is_focused.is_some() {
                    if let keyboard::Key::Character("v") = key.as_ref() {
                        state.is_pasting = None;

                        shell.capture_event();
                    }
                }

                state.is_pasting = None;
            }
            Event::Keyboard(keyboard::Event::ModifiersChanged(modifiers)) => {
                let state = state::<Renderer>(tree);

                state.keyboard_modifiers = *modifiers;
            }
            Event::InputMethod(event) => match event {
                input_method::Event::Opened | input_method::Event::Closed => {
                    let state = state::<Renderer>(tree);

                    state.preedit =
                        matches!(event, input_method::Event::Opened)
                            .then(input_method::Preedit::new);

                    shell.request_redraw();
                }
                input_method::Event::Preedit(content, selection) => {
                    let state = state::<Renderer>(tree);

                    if state.is_focused.is_some() {
                        state.preedit = Some(input_method::Preedit {
                            content: content.to_owned(),
                            selection: selection.clone(),
                            text_size: self.size,
                        });

                        shell.request_redraw();
                    }
                }
                input_method::Event::Commit(text) => {
                    let state = state::<Renderer>(tree);

                    if let Some(focus) = &mut state.is_focused {
                        let Some(on_input) = &self.on_input else {
                            return;
                        };

                        let mut editor =
                            Editor::new(&mut self.value, &mut state.cursor);
                        editor.paste(Value::new(text));

                        focus.updated_at = Instant::now();
                        state.is_pasting = None;

                        let message = (on_input)(editor.contents());
                        shell.publish(message);
                        shell.capture_event();

                        update_cache(state, &self.value);
                    }
                }
            },
            Event::Window(window::Event::Unfocused) => {
                let state = state::<Renderer>(tree);

                if let Some(focus) = &mut state.is_focused {
                    focus.is_window_focused = false;
                }
            }
            Event::Window(window::Event::Focused) => {
                let state = state::<Renderer>(tree);

                if let Some(focus) = &mut state.is_focused {
                    focus.is_window_focused = true;
                    focus.updated_at = Instant::now();

                    shell.request_redraw();
                }
            }
            Event::Window(window::Event::RedrawRequested(now)) => {
                let state = state::<Renderer>(tree);

                if let Some(focus) = &mut state.is_focused {
                    if focus.is_window_focused {
                        if matches!(
                            state.cursor.state(&self.value),
                            cursor::State::Index(_)
                        ) {
                            focus.now = *now;

                            let millis_until_redraw =
                                CURSOR_BLINK_INTERVAL_MILLIS
                                    - (*now - focus.updated_at).as_millis()
                                        % CURSOR_BLINK_INTERVAL_MILLIS;

                            shell.request_redraw_at(
                                *now + Duration::from_millis(
                                    millis_until_redraw as u64,
                                ),
                            );
                        }

                        shell.request_input_method(&self.input_method(
                            state,
                            layout,
                            &self.value,
                        ));
                    }
                }
            }
            _ => {}
        }

        let state = state::<Renderer>(tree);
        let is_disabled = self.on_input.is_none();

        let status = if is_disabled {
            Status::Disabled
        } else if state.is_focused() {
            Status::Focused {
                is_hovered: cursor.is_over(layout.bounds()),
            }
        } else if cursor.is_over(layout.bounds()) {
            Status::Hovered
        } else {
            Status::Active
        };

        if let Event::Window(window::Event::RedrawRequested(_now)) = event {
            self.last_status = Some(status);
        } else if self
            .last_status
            .is_some_and(|last_status| status != last_status)
        {
            shell.request_redraw();
        }
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        _style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        self.draw(tree, renderer, theme, layout, cursor, None, viewport);
    }

    fn mouse_interaction(
        &self,
        _state: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _viewport: &Rectangle,
        _renderer: &Renderer,
    ) -> mouse::Interaction {
        if cursor.is_over(layout.bounds()) {
            if self.on_input.is_none() {
                mouse::Interaction::Idle
            } else {
                mouse::Interaction::Text
            }
        } else {
            mouse::Interaction::default()
        }
    }
}

impl<'a, Message, Theme, Renderer> From<TextInput<'a, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: Clone + 'a,
    Theme: Catalog + 'a,
    Renderer: text::Renderer + 'a,
{
    fn from(
        text_input: TextInput<'a, Message, Theme, Renderer>,
    ) -> Element<'a, Message, Theme, Renderer> {
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
    pub size: Option<Pixels>,
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

impl From<&'static str> for Id {
    fn from(id: &'static str) -> Self {
        Self::new(id)
    }
}

impl From<String> for Id {
    fn from(id: String) -> Self {
        Self::new(id)
    }
}

/// Produces a [`Task`] that returns whether the [`TextInput`] with the given [`Id`] is focused or not.
pub fn is_focused(id: impl Into<Id>) -> Task<bool> {
    task::widget(operation::focusable::is_focused(id.into().into()))
}

/// Produces a [`Task`] that focuses the [`TextInput`] with the given [`Id`].
pub fn focus<T>(id: impl Into<Id>) -> Task<T> {
    task::effect(Action::widget(operation::focusable::focus(id.into().0)))
}

/// Produces a [`Task`] that moves the cursor of the [`TextInput`] with the given [`Id`] to the
/// end.
pub fn move_cursor_to_end<T>(id: impl Into<Id>) -> Task<T> {
    task::effect(Action::widget(operation::text_input::move_cursor_to_end(
        id.into().0,
    )))
}

/// Produces a [`Task`] that moves the cursor of the [`TextInput`] with the given [`Id`] to the
/// front.
pub fn move_cursor_to_front<T>(id: impl Into<Id>) -> Task<T> {
    task::effect(Action::widget(operation::text_input::move_cursor_to_front(
        id.into().0,
    )))
}

/// Produces a [`Task`] that moves the cursor of the [`TextInput`] with the given [`Id`] to the
/// provided position.
pub fn move_cursor_to<T>(id: impl Into<Id>, position: usize) -> Task<T> {
    task::effect(Action::widget(operation::text_input::move_cursor_to(
        id.into().0,
        position,
    )))
}

/// Produces a [`Task`] that selects all the content of the [`TextInput`] with the given [`Id`].
pub fn select_all<T>(id: impl Into<Id>) -> Task<T> {
    task::effect(Action::widget(operation::text_input::select_all(
        id.into().0,
    )))
}

/// The state of a [`TextInput`].
#[derive(Debug, Default, Clone)]
pub struct State<P: text::Paragraph> {
    value: paragraph::Plain<P>,
    placeholder: paragraph::Plain<P>,
    icon: paragraph::Plain<P>,
    is_focused: Option<Focus>,
    is_dragging: bool,
    is_pasting: Option<Value>,
    preedit: Option<input_method::Preedit>,
    last_click: Option<mouse::Click>,
    cursor: Cursor,
    keyboard_modifiers: keyboard::Modifiers,
    // TODO: Add stateful horizontal scrolling offset
}

fn state<Renderer: text::Renderer>(
    tree: &mut Tree,
) -> &mut State<Renderer::Paragraph> {
    tree.state.downcast_mut::<State<Renderer::Paragraph>>()
}

#[derive(Debug, Clone)]
struct Focus {
    updated_at: Instant,
    now: Instant,
    is_window_focused: bool,
}

impl<P: text::Paragraph> State<P> {
    /// Creates a new [`State`], representing an unfocused [`TextInput`].
    pub fn new() -> Self {
        Self::default()
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
            is_window_focused: true,
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

impl<P: text::Paragraph> operation::Focusable for State<P> {
    fn is_focused(&self) -> bool {
        State::is_focused(self)
    }

    fn focus(&mut self) {
        State::focus(self);
    }

    fn unfocus(&mut self) {
        State::unfocus(self);
    }
}

impl<P: text::Paragraph> operation::TextInput for State<P> {
    fn move_cursor_to_front(&mut self) {
        State::move_cursor_to_front(self);
    }

    fn move_cursor_to_end(&mut self) {
        State::move_cursor_to_end(self);
    }

    fn move_cursor_to(&mut self, position: usize) {
        State::move_cursor_to(self, position);
    }

    fn select_all(&mut self) {
        State::select_all(self);
    }
}

fn offset<P: text::Paragraph>(
    text_bounds: Rectangle,
    value: &Value,
    state: &State<P>,
) -> f32 {
    if state.is_focused() {
        let cursor = state.cursor();

        let focus_position = match cursor.state(value) {
            cursor::State::Index(i) => i,
            cursor::State::Selection { end, .. } => end,
        };

        let (_, offset) = measure_cursor_and_scroll_offset(
            state.value.raw(),
            text_bounds,
            focus_position,
        );

        offset
    } else {
        0.0
    }
}

fn measure_cursor_and_scroll_offset(
    paragraph: &impl text::Paragraph,
    text_bounds: Rectangle,
    cursor_index: usize,
) -> (f32, f32) {
    let grapheme_position = paragraph
        .grapheme_position(0, cursor_index)
        .unwrap_or(Point::ORIGIN);

    let offset = ((grapheme_position.x + 5.0) - text_bounds.width).max(0.0);

    (grapheme_position.x, offset)
}

/// Computes the position of the text cursor at the given X coordinate of
/// a [`TextInput`].
fn find_cursor_position<P: text::Paragraph>(
    text_bounds: Rectangle,
    value: &Value,
    state: &State<P>,
    x: f32,
) -> Option<usize> {
    let offset = offset(text_bounds, value, state);
    let value = value.to_string();

    let char_offset = state
        .value
        .raw()
        .hit_test(Point::new(x + offset, text_bounds.height / 2.0))
        .map(text::Hit::cursor)?;

    Some(
        unicode_segmentation::UnicodeSegmentation::graphemes(
            &value[..char_offset.min(value.len())],
            true,
        )
        .count(),
    )
}

fn replace_paragraph<Renderer>(
    renderer: &Renderer,
    state: &mut State<Renderer::Paragraph>,
    layout: Layout<'_>,
    value: &Value,
    font: Option<Renderer::Font>,
    text_size: Option<Pixels>,
    line_height: text::LineHeight,
) where
    Renderer: text::Renderer,
{
    let font = font.unwrap_or_else(|| renderer.default_font());
    let text_size = text_size.unwrap_or_else(|| renderer.default_size());

    let mut children_layout = layout.children();
    let text_bounds = children_layout.next().unwrap().bounds();

    state.value = paragraph::Plain::new(Text {
        font,
        line_height,
        content: value.to_string(),
        bounds: Size::new(f32::INFINITY, text_bounds.height),
        size: text_size,
        align_x: text::Alignment::Default,
        align_y: alignment::Vertical::Center,
        shaping: text::Shaping::Advanced,
        wrapping: text::Wrapping::default(),
    });
}

const CURSOR_BLINK_INTERVAL_MILLIS: u128 = 500;

/// The possible status of a [`TextInput`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    /// The [`TextInput`] can be interacted with.
    Active,
    /// The [`TextInput`] is being hovered.
    Hovered,
    /// The [`TextInput`] is focused.
    Focused {
        /// Whether the [`TextInput`] is hovered, while focused.
        is_hovered: bool,
    },
    /// The [`TextInput`] cannot be interacted with.
    Disabled,
}

/// The appearance of a text input.
#[derive(Debug, Clone, Copy, PartialEq)]
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

/// The theme catalog of a [`TextInput`].
pub trait Catalog: Sized {
    /// The item class of the [`Catalog`].
    type Class<'a>;

    /// The default class produced by the [`Catalog`].
    fn default<'a>() -> Self::Class<'a>;

    /// The [`Style`] of a class with the given status.
    fn style(&self, class: &Self::Class<'_>, status: Status) -> Style;
}

/// A styling function for a [`TextInput`].
///
/// This is just a boxed closure: `Fn(&Theme, Status) -> Style`.
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

/// The default style of a [`TextInput`].
pub fn default(theme: &Theme, status: Status) -> Style {
    let palette = theme.extended_palette();

    let active = Style {
        background: Background::Color(palette.background.base.color),
        border: Border {
            radius: 2.0.into(),
            width: 1.0,
            color: palette.background.strongest.color,
        },
        icon: palette.background.weak.text,
        placeholder: palette.background.strongest.color,
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
        Status::Focused { .. } => Style {
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

fn alignment_offset(
    text_bounds_width: f32,
    text_min_width: f32,
    alignment: alignment::Horizontal,
) -> f32 {
    if text_min_width > text_bounds_width {
        0.0
    } else {
        match alignment {
            alignment::Horizontal::Left => 0.0,
            alignment::Horizontal::Center => {
                (text_bounds_width - text_min_width) / 2.0
            }
            alignment::Horizontal::Right => text_bounds_width - text_min_width,
        }
    }
}
