//! Allow your users to perform actions by pressing a button.
//!
//! A [`Button`] has some local [`State`].
//!
//! [`Button`]: struct.Button.html
//! [`State`]: struct.State.html

use crate::{Background, Length};

/// A generic widget that produces a message when clicked.
#[allow(missing_docs)]
#[derive(Debug)]
pub struct Button<'a, Message, Element> {
    pub state: &'a mut State,
    pub content: Element,
    pub on_press: Option<Message>,
    pub width: Length,
    pub min_width: u32,
    pub padding: u16,
    pub background: Option<Background>,
    pub border_radius: u16,
}

impl<'a, Message, Element> Button<'a, Message, Element> {
    /// Creates a new [`Button`] with some local [`State`] and the given
    /// content.
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
    /// Whether the [`Button`] is currently being pressed.
    ///
    /// [`Button`]: struct.Button.html
    pub is_pressed: bool,
}

impl State {
    /// Creates a new [`State`].
    ///
    /// [`State`]: struct.State.html
    pub fn new() -> State {
        State::default()
    }
}
