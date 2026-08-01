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
use crate::core::keyboard;
use crate::core::layout;
use crate::core::mouse;
use crate::core::renderer;
use crate::core::shell;
use crate::core::text;
use crate::core::text::editor;
use crate::core::text::input;
use crate::core::widget;
use crate::core::widget::operation::{self, Focusable, Operation};
use crate::core::widget::tree::{self, Tree};
use crate::core::window;
use crate::core::{
    Background, Border, Color, Element, Event, Layout, Length, Padding, Pixels, Rectangle, Shell,
    Size, Theme, Widget,
};

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
pub struct TextInput<'a, Message, Theme = crate::Theme, Renderer = crate::Renderer>
where
    Theme: Catalog,
    Renderer: text::Renderer,
{
    id: Option<widget::Id>,
    placeholder: text::Fragment<'a>,
    value: text::Fragment<'a>,
    is_secure: bool,
    font: Option<Renderer::Font>,
    width: Length,
    height: Length,
    padding: Padding,
    size: Option<Pixels>,
    line_height: text::LineHeight,
    alignment: text::Alignment,
    multiline: Option<text::Wrapping>,
    on_input: Option<Box<dyn Fn(String) -> Message + 'a>>,
    on_paste: Option<Box<dyn Fn(String) -> Message + 'a>>,
    on_submit: Option<Message>,
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
    pub fn new(
        placeholder: impl text::IntoFragment<'a>,
        value: impl text::IntoFragment<'a>,
    ) -> Self {
        TextInput {
            id: None,
            placeholder: placeholder.into_fragment(),
            value: value.into_fragment(),
            is_secure: false,
            font: None,
            width: Length::Fill,
            height: Length::Fit,
            padding: DEFAULT_PADDING,
            size: None,
            line_height: text::LineHeight::default(),
            alignment: text::Alignment::Default,
            multiline: None,
            on_input: None,
            on_paste: None,
            on_submit: None,
            class: Theme::default(),
            last_status: None,
        }
    }

    /// Sets the [`widget::Id`] of the [`TextInput`].
    pub fn id(mut self, id: impl Into<widget::Id>) -> Self {
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
    pub fn on_input(mut self, on_input: impl Fn(String) -> Message + 'a) -> Self {
        self.on_input = Some(Box::new(on_input));
        self
    }

    /// Sets the message that should be produced when some text is typed into
    /// the [`TextInput`], if `Some`.
    ///
    /// If `None`, the [`TextInput`] will be disabled.
    pub fn on_input_maybe(mut self, on_input: Option<impl Fn(String) -> Message + 'a>) -> Self {
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
    pub fn on_paste(mut self, on_paste: impl Fn(String) -> Message + 'a) -> Self {
        self.on_paste = Some(Box::new(on_paste));
        self
    }

    /// Sets the message that should be produced when some text is pasted into
    /// the [`TextInput`], if `Some`.
    pub fn on_paste_maybe(mut self, on_paste: Option<impl Fn(String) -> Message + 'a>) -> Self {
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
    pub fn line_height(mut self, line_height: impl Into<text::LineHeight>) -> Self {
        self.line_height = line_height.into();
        self
    }

    /// Sets the horizontal alignment of the [`TextInput`].
    pub fn align_x(mut self, alignment: impl Into<text::Alignment>) -> Self {
        self.alignment = alignment.into();
        self
    }

    /// Sets the multiline behavior of the [`TextInput`].
    ///
    /// `None` will behave as a single line input.
    pub fn multiline(mut self, wrapping: Option<text::Wrapping>) -> Self {
        self.multiline = wrapping;
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
}

impl<Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for TextInput<'_, Message, Theme, Renderer>
where
    Message: Clone,
    Theme: Catalog,
    Renderer: text::Renderer + 'static,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State<Renderer>>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State::<Renderer>::new())
    }

    fn size(&self) -> Size<Length> {
        Size {
            width: self.width,
            height: Length::Shrink,
        }
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let state = tree.state.downcast_mut::<State<Renderer>>();

        if state.value != self.value
            && state
                .transaction
                .as_ref()
                .is_none_or(shell::Tracking::is_processed)
        {
            state.input.overwrite(self.value.as_ref());
            state.value = self.value.clone().into_owned();
        }

        state.input.layout(
            renderer,
            limits,
            input::Layout {
                width: self.width,
                height: self.height,
                padding: self.padding,
                placeholder: self.placeholder.as_ref(),
                font: self.font,
                size: self.size,
                line_height: self.line_height,
                alignment: self.alignment,
                multiline: self.multiline,
            },
        )
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        _renderer: &Renderer,
        operation: &mut dyn Operation,
    ) {
        let state = tree.state.downcast_mut::<State<Renderer>>();

        operation.text_input(self.id.as_ref(), layout.bounds(), state);
        operation.focusable(self.id.as_ref(), layout.bounds(), state);
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _renderer: &Renderer,
        shell: &mut Shell<'_, Message>,
        _viewport: &Rectangle,
    ) {
        let state = state::<Renderer>(tree);
        let is_disabled = self.on_input.is_none();

        if let Some(on_input) = &self.on_input {
            let edit = state
                .input
                .update(event, layout.bounds(), cursor, shell, |key_press| {
                    if let Some(on_submit) = &self.on_submit
                        && key_press.modified_key
                            == keyboard::Key::Named(keyboard::key::Named::Enter)
                    {
                        return Some(editor::Binding::Custom(on_submit.clone()));
                    }

                    editor::Binding::from_key_press(key_press)
                });

            if let Some(edit) = edit {
                let on_input = if let Some(on_paste) = &self.on_paste
                    && edit.is_paste
                {
                    on_paste
                } else {
                    on_input
                };

                state.value = state.input.value();
                state.transaction = Some(shell.publish_and_track(on_input(state.value.clone())));
            }
        }

        let status = if is_disabled {
            Status::Disabled
        } else if state.input.is_focused() {
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

            shell.request_input_method(
                &state
                    .input
                    .input_method(layout.bounds().shrink(self.padding).position()),
            );
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
        _cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let state = tree.state.downcast_ref::<State<Renderer>>();
        let style = theme.style(&self.class, self.last_status.unwrap_or(Status::Disabled));
        let bounds = layout.bounds();

        renderer.fill_quad(
            renderer::Quad {
                bounds,
                border: style.border,
                ..renderer::Quad::default()
            },
            style.background,
        );

        state.input.draw(
            renderer,
            bounds,
            *viewport,
            input::Style {
                value: style.value,
                selection: style.selection,
                placeholder: style.placeholder,
            },
        );
    }

    fn mouse_interaction(
        &self,
        _tree: &Tree,
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
    Renderer: text::Renderer + 'static,
{
    fn from(
        text_input: TextInput<'a, Message, Theme, Renderer>,
    ) -> Element<'a, Message, Theme, Renderer> {
        Element::new(text_input)
    }
}

/// The state of a [`TextInput`].
struct State<R: text::Renderer> {
    input: text::Input<R>,
    value: String,
    transaction: Option<shell::Tracking>,
}

fn state<Renderer: text::Renderer + 'static>(tree: &mut Tree) -> &mut State<Renderer> {
    tree.state.downcast_mut::<State<Renderer>>()
}

impl<R: text::Renderer> State<R> {
    /// Creates a new [`State`], representing an unfocused [`TextInput`].
    fn new() -> Self {
        Self {
            input: text::Input::new(),
            value: String::new(),
            transaction: None,
        }
    }
}

impl<R: text::Renderer> operation::Focusable for State<R> {
    fn is_focused(&self) -> bool {
        self.input.is_focused()
    }

    fn focus(&mut self) {
        self.input.focus();
    }

    fn unfocus(&mut self) {
        self.input.unfocus();
    }
}

impl<R: text::Renderer> operation::TextInput for State<R> {
    fn text(&self) -> text::Fragment<'_> {
        if self.input.is_empty() {
            text::Fragment::Borrowed(self.input.placeholder())
        } else {
            text::Fragment::Owned(self.input.value())
        }
    }

    fn move_cursor_to_front(&mut self) {
        self.input.move_cursor_to_front();
    }

    fn move_cursor_to_end(&mut self) {
        self.input.move_cursor_to_end();
    }

    fn move_cursor_to(&mut self, position: text::Position) {
        self.input.move_cursor_to(position);
    }

    fn select_all(&mut self) {
        self.input.select_all();
    }

    fn select_range(&mut self, start: text::Position, end: text::Position) {
        self.input.select_range(start, end);
    }
}

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
    let palette = theme.palette();

    let active = Style {
        background: Background::Color(palette.background.base.color),
        border: Border {
            radius: 2.0.into(),
            width: 1.0,
            color: palette.background.strong.color,
        },
        placeholder: palette.secondary.base.color,
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
            placeholder: palette.background.strongest.color,
            ..active
        },
    }
}
