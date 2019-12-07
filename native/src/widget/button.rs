//! Allow your users to perform actions by pressing a button.
//!
//! A [`Button`] has some local [`State`].
//!
//! [`Button`]: struct.Button.html
//! [`State`]: struct.State.html
use crate::{
    input::{mouse, ButtonState},
    layout, Element, Event, Hasher, Layout, Length, Point, Rectangle, Widget,
};
use std::hash::Hash;

/// A generic widget that produces a message when pressed.
///
/// ```
/// # use iced_native::{button, Text, TextRole};
/// #
/// # type Button<'a, Message> =
/// #     iced_native::Button<'a, Message, iced_native::renderer::Null, <iced_native::renderer::Null as iced_native::button::Renderer>::WidgetStyle>;
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
pub struct Button<'a, Message, Renderer, Style> {
    state: &'a mut State,
    content: Element<'a, Message, Renderer>,
    on_press: Option<Message>,
    width: Length,
    height: Length,
    min_width: u32,
    min_height: u32,
    padding: u16,
    style: Style,
}

impl<'a, Message, Renderer, Style> Button<'a, Message, Renderer, Style> {
    /// Creates a new [`Button`] with some local [`State`] and the given
    /// content.
    ///
    /// [`Button`]: type.Button.html
    /// [`State`]: struct.State.html
    pub fn new<E>(state: &'a mut State, content: E) -> Self
    where
        E: Into<Element<'a, Message, Renderer>>,
        Style: Default,
    {
        Button {
            state,
            content: content.into(),
            on_press: None,
            width: Length::Shrink,
            height: Length::Shrink,
            min_width: 0,
            min_height: 0,
            style: Style::default(),
            padding: 10,
        }
    }

    /// Creates a new [`Button`] with some local [`State`] and the given
    /// content and a custom `style`.
    ///
    /// [`Button`]: type.Button.html
    /// [`State`]: struct.State.html
    /// [`Palette`]: ../struct.Palette.html
    pub fn new_with_style<E>(state: &'a mut State, content: E, style: Style) -> Self
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
            style,
            padding: 10,
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

    /// Sets the min_width of the [`Button`].
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

    /// Changes the style of the [`Button`].
    ///
    /// [`Button`]: struct.Button.html
    pub fn change_style(mut self, f: impl FnOnce(&mut Style)) -> Self {
        f(&mut self.style);
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

impl<'a, Message, Renderer, Style> Widget<Message, Renderer>
    for Button<'a, Message, Renderer, Style>
where
    Renderer: self::Renderer<WidgetStyle = Style>,
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

        content.bounds.x = padding;
        content.bounds.y = padding;

        let size = limits.resolve(content.size()).pad(padding);

        layout::Node::with_children(size, vec![content])
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
            &self.style,
            content,
        )
    }

    fn hash_layout(&self, state: &mut Hasher) {
        self.width.hash(state);
        self.content.hash_layout(state);
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
}

/// The renderer of a [`Button`].
///
/// Your [renderer] will need to implement this trait before being
/// able to use a [`Button`] in your user interface.
///
/// [`Button`]: struct.Button.html
/// [renderer]: ../../renderer/index.html
pub trait Renderer: crate::Renderer + Sized {
    /// Struct that consists of all style options the renderer supports for
    /// [`Button`].
    ///
    /// [`Button`]: struct.Button.html
    type WidgetStyle;

    /// Draws a [`Button`].
    ///
    /// [`Button`]: struct.Button.html
    fn draw(
        &mut self,
        bounds: Rectangle,
        cursor_position: Point,
        is_pressed: bool,
        style: &Self::WidgetStyle,
        content: Self::Output,
    ) -> Self::Output;
}

impl<'a, Message, Renderer, Style> From<Button<'a, Message, Renderer, Style>>
    for Element<'a, Message, Renderer>
where
    Renderer: 'static + self::Renderer<WidgetStyle = Style>,
    Message: 'static + Clone,
    Style: 'static,
{
    fn from(
        button: Button<'a, Message, Renderer, Style>,
    ) -> Element<'a, Message, Renderer> {
        Element::new(button)
    }
}
