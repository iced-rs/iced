//! Allow your users to perform actions by pressing a button.
//!
//! A [`Button`] has some local [`State`].
//!
//! [`Button`]: struct.Button.html
//! [`State`]: struct.State.html
use crate::{
    input::{mouse, ButtonState},
    layout, Background, Element, Event, Hasher, Layout, Length, Point,
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
pub struct Button<'a, Message, Renderer> {
    state: &'a mut State,
    content: Element<'a, Message, Renderer>,
    on_press: Option<Message>,
    width: Length,
    min_width: u32,
    padding: u16,
    background: Option<Background>,
    border_radius: u16,
}

impl<'a, Message, Renderer> Button<'a, Message, Renderer> {
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
            min_width: 0,
            padding: 0,
            background: None,
            border_radius: 0,
        }
    }

    /// Sets the width of the [`Button`].
    ///
    /// [`Button`]: struct.Button.html
    pub fn width(mut self, width: Length) -> Self {
        self.width = width;
        self
    }

    /// Sets the minimum width of the [`Button`].
    ///
    /// [`Button`]: struct.Button.html
    pub fn min_width(mut self, min_width: u32) -> Self {
        self.min_width = min_width;
        self
    }

    /// Sets the padding of the [`Button`].
    ///
    /// [`Button`]: struct.Button.html
    pub fn padding(mut self, padding: u16) -> Self {
        self.padding = padding;
        self
    }

    /// Sets the [`Background`] of the [`Button`].
    ///
    /// [`Button`]: struct.Button.html
    /// [`Background`]: ../../struct.Background.html
    pub fn background(mut self, background: Background) -> Self {
        self.background = Some(background);
        self
    }

    /// Sets the border radius of the [`Button`].
    ///
    /// [`Button`]: struct.Button.html
    pub fn border_radius(mut self, border_radius: u16) -> Self {
        self.border_radius = border_radius;
        self
    }

    /// Sets the message that will be produced when the [`Button`] is pressed.
    ///
    /// [`Button`]: struct.Button.html
    pub fn on_press(mut self, msg: Message) -> Self {
        self.on_press = Some(msg);
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

impl<'a, Message, Renderer> Widget<Message, Renderer>
    for Button<'a, Message, Renderer>
where
    Renderer: self::Renderer,
    Message: Clone,
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
        let padding = f32::from(self.padding);
        let limits = limits
            .min_width(self.min_width)
            .width(self.width)
            .height(Length::Shrink)
            .pad(padding);

        let mut content = self.content.layout(renderer, &limits);

        content.bounds.x = padding;
        content.bounds.y = padding;

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
    ) {
        match event {
            Event::Mouse(mouse::Event::Input {
                button: mouse::Button::Left,
                state,
            }) => {
                if let Some(on_press) = self.on_press.clone() {
                    let bounds = layout.bounds();

                    match state {
                        ButtonState::Pressed => {
                            self.state.is_pressed =
                                bounds.contains(cursor_position);
                        }
                        ButtonState::Released => {
                            let is_clicked = self.state.is_pressed
                                && bounds.contains(cursor_position);

                            self.state.is_pressed = false;

                            if is_clicked {
                                messages.push(on_press);
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }

    fn draw(
        &self,
        renderer: &mut Renderer,
        layout: Layout<'_>,
        cursor_position: Point,
    ) -> Renderer::Output {
        let content = self.content.draw(
            renderer,
            layout.children().next().unwrap(),
            cursor_position,
        );

        renderer.draw(
            layout.bounds(),
            cursor_position,
            self.state.is_pressed,
            self.background,
            self.border_radius,
            content,
        )
    }

    fn hash_layout(&self, state: &mut Hasher) {
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
    /// Draws a [`Button`].
    ///
    /// [`Button`]: struct.Button.html
    fn draw(
        &mut self,
        bounds: Rectangle,
        cursor_position: Point,
        is_pressed: bool,
        background: Option<Background>,
        border_radius: u16,
        content: Self::Output,
    ) -> Self::Output;
}

impl<'a, Message, Renderer> From<Button<'a, Message, Renderer>>
    for Element<'a, Message, Renderer>
where
    Renderer: 'static + self::Renderer,
    Message: 'static + Clone,
{
    fn from(
        button: Button<'a, Message, Renderer>,
    ) -> Element<'a, Message, Renderer> {
        Element::new(button)
    }
}
