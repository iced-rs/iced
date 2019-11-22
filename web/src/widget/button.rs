//! Allow your users to perform actions by pressing a button.
//!
//! A [`Button`] has some local [`State`].
//!
//! [`Button`]: struct.Button.html
//! [`State`]: struct.State.html
use crate::{Background, Bus, Element, Length, Widget};

use dodrio::bumpalo;

/// A generic widget that produces a message when pressed.
///
/// ```
/// # use iced_web::{button, Button, Text};
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
pub struct Button<'a, Message> {
    content: Element<'a, Message>,
    on_press: Option<Message>,
    width: Length,
    min_width: u32,
    padding: u16,
    background: Option<Background>,
    border_radius: u16,
}

impl<'a, Message> Button<'a, Message> {
    /// Creates a new [`Button`] with some local [`State`] and the given
    /// content.
    ///
    /// [`Button`]: struct.Button.html
    /// [`State`]: struct.State.html
    pub fn new<E>(_state: &'a mut State, content: E) -> Self
    where
        E: Into<Element<'a, Message>>,
    {
        Button {
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
pub struct State;

impl State {
    /// Creates a new [`State`].
    ///
    /// [`State`]: struct.State.html
    pub fn new() -> State {
        State::default()
    }
}

impl<'a, Message> Widget<Message> for Button<'a, Message>
where
    Message: 'static + Copy,
{
    fn node<'b>(
        &self,
        bump: &'b bumpalo::Bump,
        bus: &Bus<Message>,
    ) -> dodrio::Node<'b> {
        use dodrio::builder::*;

        let mut node =
            button(bump).children(vec![self.content.node(bump, bus)]);

        if let Some(on_press) = self.on_press {
            let event_bus = bus.clone();

            node = node.on("click", move |root, vdom, _event| {
                event_bus.publish(on_press, root);

                vdom.schedule_render();
            });
        }

        // TODO: Complete styling

        node.finish()
    }
}

impl<'a, Message> From<Button<'a, Message>> for Element<'a, Message>
where
    Message: 'static + Copy,
{
    fn from(button: Button<'a, Message>) -> Element<'a, Message> {
        Element::new(button)
    }
}
