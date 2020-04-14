//! Allow your users to perform actions by pressing a button.
//!
//! A [`Button`] has some local [`State`].
//!
//! [`Button`]: struct.Button.html
//! [`State`]: struct.State.html
use crate::{
    layout, mouse, Clipboard, Element, Event, Hasher, Layout, Length, Point,
    Rectangle, Widget,
};
use std::hash::Hash;

/// A generic widget that produces a message when pressed.
///
/// ```
/// # use iced_native::{button, Text};
/// #
/// # type Button<'a, Message> =
/// #     iced_native::Button<'a, Message, iced_native::renderer::Null>;
/// #
/// enum Message {
///     ButtonPressed,
/// }
///
/// let mut state = button::State::new();
/// let button = Button::new(&mut state, Text::new("Press me!"))
///     .on_press(Message::ButtonPressed);
/// ```
#[allow(missing_debug_implementations)]
pub struct Button<'a, Message, Renderer: self::Renderer> {
    state: &'a mut State,
    content: Element<'a, Message, Renderer>,
    on_press: Option<Message>,
    width: Length,
    height: Length,
    min_width: u32,
    min_height: u32,
    padding: u16,
    style: Renderer::Style,
}

impl<'a, Message, Renderer> Button<'a, Message, Renderer>
where
    Renderer: self::Renderer,
{
    /// Creates a new [`Button`] with some local [`State`] and the given
    /// content.
    ///
    /// [`Button`]: struct.Button.html
    /// [`State`]: struct.State.html
    pub fn new<E>(state: &'a mut State, content: E) -> Self
    where
        E: Into<Element<'a, Message, Renderer>>,
    {
        Button {
            state,
            content: content.into(),
            on_press: None,
            width: Length::Shrink,
            height: Length::Shrink,
            min_width: 0,
            min_height: 0,
            padding: Renderer::DEFAULT_PADDING,
            style: Renderer::Style::default(),
        }
    }

    /// Sets the width of the [`Button`].
    ///
    /// [`Button`]: struct.Button.html
    pub fn width(mut self, width: Length) -> Self {
        self.width = width;
        self
    }

    /// Sets the height of the [`Button`].
    ///
    /// [`Button`]: struct.Button.html
    pub fn height(mut self, height: Length) -> Self {
        self.height = height;
        self
    }

    /// Sets the minimum width of the [`Button`].
    ///
    /// [`Button`]: struct.Button.html
    pub fn min_width(mut self, min_width: u32) -> Self {
        self.min_width = min_width;
        self
    }

    /// Sets the minimum height of the [`Button`].
    ///
    /// [`Button`]: struct.Button.html
    pub fn min_height(mut self, min_height: u32) -> Self {
        self.min_height = min_height;
        self
    }

    /// Sets the padding of the [`Button`].
    ///
    /// [`Button`]: struct.Button.html
    pub fn padding(mut self, padding: u16) -> Self {
        self.padding = padding;
        self
    }

    /// Sets the message that will be produced when the [`Button`] is pressed.
    ///
    /// [`Button`]: struct.Button.html
    pub fn on_press(mut self, msg: Message) -> Self {
        self.on_press = Some(msg);
        self
    }

    /// Sets the style of the [`Button`].
    ///
    /// [`Button`]: struct.Button.html
    pub fn style(mut self, style: impl Into<Renderer::Style>) -> Self {
        self.style = style.into();
        self
    }
}

/// The local state of a [`Button`].
///
/// [`Button`]: struct.Button.html
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct State {
    is_pressed: bool,
}

impl State {
    /// Creates a new [`State`].
    ///
    /// [`State`]: struct.State.html
    pub fn new() -> State {
        State::default()
    }
}

impl<'a, Message, Renderer> Widget<'a, Message, Renderer>
    for Button<'a, Message, Renderer>
where
    Renderer: self::Renderer,
    Message: Clone,
{
    fn width(&self) -> Length {
        self.width
    }

    fn height(&self) -> Length {
        self.height
    }

    fn layout(
        &self,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let padding = f32::from(self.padding);
        let limits = limits
            .min_width(self.min_width)
            .min_height(self.min_height)
            .width(self.width)
            .height(self.height)
            .pad(padding);

        let mut content = self.content.layout(renderer, &limits);
        content.move_to(Point::new(padding, padding));

        let size = limits.resolve(content.size()).pad(padding);

        layout::Node::with_children(size, vec![content])
    }

    fn on_event(
        &mut self,
        event: Event,
        layout: Layout<'_>,
        cursor_position: Point,
        messages: &mut Vec<Message>,
        _renderer: &Renderer,
        _clipboard: Option<&dyn Clipboard>,
    ) {
        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                if self.on_press.is_some() {
                    let bounds = layout.bounds();

                    self.state.is_pressed = bounds.contains(cursor_position);
                }
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                if let Some(on_press) = self.on_press.clone() {
                    let bounds = layout.bounds();

                    let is_clicked = self.state.is_pressed
                        && bounds.contains(cursor_position);

                    self.state.is_pressed = false;

                    if is_clicked {
                        messages.push(on_press);
                    }
                }
            }
            _ => {}
        }
    }

    fn draw(
        &self,
        renderer: &mut Renderer,
        defaults: &Renderer::Defaults,
        layout: Layout<'_>,
        cursor_position: Point,
    ) -> Renderer::Output {
        renderer.draw(
            defaults,
            layout.bounds(),
            cursor_position,
            self.on_press.is_none(),
            self.state.is_pressed,
            &self.style,
            &self.content,
            layout.children().next().unwrap(),
        )
    }

    fn hash_layout(&self, state: &mut Hasher) {
        struct Marker;
        std::any::TypeId::of::<Marker>().hash(state);

        self.width.hash(state);
        self.content.hash_layout(state);
    }
}

/// The renderer of a [`Button`].
///
/// Your [renderer] will need to implement this trait before being
/// able to use a [`Button`] in your user interface.
///
/// [`Button`]: struct.Button.html
/// [renderer]: ../../renderer/index.html
pub trait Renderer: crate::Renderer + Sized {
    /// The default padding of a [`Button`].
    ///
    /// [`Button`]: struct.Button.html
    const DEFAULT_PADDING: u16;

    /// The style supported by this renderer.
    type Style: Default;

    /// Draws a [`Button`].
    ///
    /// [`Button`]: struct.Button.html
    fn draw<Message>(
        &mut self,
        defaults: &Self::Defaults,
        bounds: Rectangle,
        cursor_position: Point,
        is_disabled: bool,
        is_pressed: bool,
        style: &Self::Style,
        content: &Element<'_, Message, Self>,
        content_layout: Layout<'_>,
    ) -> Self::Output;
}

impl<'a, Message, Renderer> From<Button<'a, Message, Renderer>>
    for Element<'a, Message, Renderer>
where
    Renderer: 'a + self::Renderer,
    Message: 'a + Clone,
{
    fn from(
        button: Button<'a, Message, Renderer>,
    ) -> Element<'a, Message, Renderer> {
        Element::new(button)
    }
}
