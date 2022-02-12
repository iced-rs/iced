use crate::widget::{Element, Tree, Widget};

use iced_native::event::{self, Event};
use iced_native::layout::{self, Layout};
use iced_native::mouse;
use iced_native::renderer;
use iced_native::text;
use iced_native::widget::text_input;
use iced_native::{
    Clipboard, Hasher, Length, Padding, Point, Rectangle, Shell,
};

pub use iced_style::text_input::StyleSheet;

use std::any::{self, Any};

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
/// ![Text input drawn by `iced_wgpu`](https://github.com/iced-rs/iced/blob/7760618fb112074bc40b148944521f312152012a/docs/images/text_input.png?raw=true)
#[allow(missing_debug_implementations)]
pub struct TextInput<'a, Message, Renderer: text::Renderer> {
    placeholder: String,
    value: text_input::Value,
    is_secure: bool,
    font: Renderer::Font,
    width: Length,
    padding: Padding,
    size: Option<u16>,
    on_change: Box<dyn Fn(String) -> Message + 'a>,
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
    pub fn new<F>(placeholder: &str, value: &str, on_change: F) -> Self
    where
        F: 'a + Fn(String) -> Message,
    {
        TextInput {
            placeholder: String::from(placeholder),
            value: text_input::Value::new(value),
            is_secure: false,
            font: Default::default(),
            width: Length::Fill,
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
}

impl<'a, Message, Renderer> Widget<Message, Renderer>
    for TextInput<'a, Message, Renderer>
where
    Message: Clone,
    Renderer: iced_native::text::Renderer,
{
    fn tag(&self) -> any::TypeId {
        any::TypeId::of::<text_input::State>()
    }

    fn state(&self) -> Box<dyn Any> {
        Box::new(text_input::State::new())
    }

    fn diff(&self, _tree: &mut Tree) {}

    fn children_state(&self) -> Vec<Tree> {
        Vec::new()
    }

    fn width(&self) -> Length {
        self.width
    }

    fn height(&self) -> Length {
        Length::Shrink
    }

    fn hash_layout(&self, state: &mut Hasher) {
        text_input::hash_layout(state, self.width, self.padding, self.size);
    }

    fn layout(
        &self,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        text_input::layout(
            renderer,
            limits,
            self.width,
            self.padding,
            self.size,
        )
    }

    fn on_event(
        &mut self,
        tree: &mut Tree,
        event: Event,
        layout: Layout<'_>,
        cursor_position: Point,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
    ) -> event::Status {
        text_input::update(
            event,
            layout,
            cursor_position,
            renderer,
            clipboard,
            shell,
            &mut self.value,
            self.size,
            &self.font,
            self.is_secure,
            self.on_change.as_ref(),
            &self.on_submit,
            || tree.state.downcast_mut::<text_input::State>(),
        )
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        _style: &renderer::Style,
        layout: Layout<'_>,
        cursor_position: Point,
        _viewport: &Rectangle,
    ) {
        text_input::draw(
            renderer,
            layout,
            cursor_position,
            tree.state.downcast_ref::<text_input::State>(),
            &self.value,
            &self.placeholder,
            self.size,
            &self.font,
            self.is_secure,
            self.style_sheet.as_ref(),
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
        text_input::mouse_interaction(layout, cursor_position)
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
