//! Allow your users to perform actions by pressing a button.
//!
//! A [`Button`] has some local [`State`].
//!
//! [`Button`]: struct.Button.html
//! [`State`]: struct.State.html

use crate::{Align, Length};

/// A generic widget that produces a message when clicked.
///
/// # Example
///
/// ```
/// use iced_core::{button, Button};
///
/// pub enum Message {
///     ButtonClicked,
/// }
///
/// let state = &mut button::State::new();
///
/// Button::new(state, "Click me!")
///     .on_press(Message::ButtonClicked);
/// ```
///
/// ![Button drawn by Coffee's renderer](https://github.com/hecrj/coffee/blob/bda9818f823dfcb8a7ad0ff4940b4d4b387b5208/images/ui/button.png?raw=true)
pub struct Button<'a, Message> {
    /// The current state of the button
    pub state: &'a mut State,

    /// The label of the button
    pub label: String,

    /// The message to produce when the button is pressed
    pub on_press: Option<Message>,

    pub class: Class,

    pub width: Length,

    pub align_self: Option<Align>,
}

impl<'a, Message> std::fmt::Debug for Button<'a, Message>
where
    Message: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Button")
            .field("state", &self.state)
            .field("label", &self.label)
            .field("on_press", &self.on_press)
            .finish()
    }
}

impl<'a, Message> Button<'a, Message> {
    /// Creates a new [`Button`] with some local [`State`] and the given label.
    ///
    /// [`Button`]: struct.Button.html
    /// [`State`]: struct.State.html
    pub fn new(state: &'a mut State, label: &str) -> Self {
        Button {
            state,
            label: String::from(label),
            on_press: None,
            class: Class::Primary,
            width: Length::Shrink,
            align_self: None,
        }
    }

    /// Sets the width of the [`Button`].
    ///
    /// [`Button`]: struct.Button.html
    pub fn width(mut self, width: Length) -> Self {
        self.width = width;
        self
    }

    /// Sets the alignment of the [`Button`] itself.
    ///
    /// This is useful if you want to override the default alignment given by
    /// the parent container.
    ///
    /// [`Button`]: struct.Button.html
    pub fn align_self(mut self, align: Align) -> Self {
        self.align_self = Some(align);
        self
    }

    /// Sets the [`Class`] of the [`Button`].
    ///
    ///
    /// [`Button`]: struct.Button.html
    /// [`Class`]: enum.Class.html
    pub fn class(mut self, class: Class) -> Self {
        self.class = class;
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

/// The type of a [`Button`].
///
/// ![Different buttons drawn by the built-in renderer in Coffee](https://github.com/hecrj/coffee/blob/bda9818f823dfcb8a7ad0ff4940b4d4b387b5208/images/ui/button_classes.png?raw=true)
///
/// [`Button`]: struct.Button.html
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Class {
    /// The [`Button`] performs the main action.
    ///
    /// [`Button`]: struct.Button.html
    Primary,

    /// The [`Button`] performs an alternative action.
    ///
    /// [`Button`]: struct.Button.html
    Secondary,

    /// The [`Button`] performs a productive action.
    ///
    /// [`Button`]: struct.Button.html
    Positive,
}
