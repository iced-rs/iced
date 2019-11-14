//! Allow your users to perform actions by pressing a button.
//!
//! A [`Button`] has some local [`State`].
//!
//! [`Button`]: struct.Button.html
//! [`State`]: struct.State.html

use crate::{Background, Length};

/// A generic widget that produces a message when clicked.
pub struct Button<'a, Message, Element> {
    /// The current state of the button
    pub state: &'a mut State,

    pub content: Element,

    /// The message to produce when the button is pressed
    pub on_press: Option<Message>,

    pub width: Length,

    pub padding: u16,

    pub background: Option<Background>,

    pub border_radius: u16,
}

impl<'a, Message, Element> std::fmt::Debug for Button<'a, Message, Element>
where
    Message: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Button")
            .field("state", &self.state)
            .field("on_press", &self.on_press)
            .finish()
    }
}

impl<'a, Message, Element> Button<'a, Message, Element> {
    /// Creates a new [`Button`] with some local [`State`] and the given label.
    ///
    /// [`Button`]: struct.Button.html
    /// [`State`]: struct.State.html
    pub fn new<E>(state: &'a mut State, content: E) -> Self
    where
        E: Into<Element>,
    {
        Button {
            state,
            content: content.into(),
            on_press: None,
            width: Length::Shrink,
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

    pub fn padding(mut self, padding: u16) -> Self {
        self.padding = padding;
        self
    }

    pub fn background(mut self, background: Background) -> Self {
        self.background = Some(background);
        self
    }

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
    pub is_pressed: bool,
}

impl State {
    /// Creates a new [`State`].
    ///
    /// [`State`]: struct.State.html
    pub fn new() -> State {
        State::default()
    }

    /// Returns whether the associated [`Button`] is currently being pressed or
    /// not.
    ///
    /// [`Button`]: struct.Button.html
    pub fn is_pressed(&self) -> bool {
        self.is_pressed
    }
}
