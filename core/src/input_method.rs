//! Listen to input method events.
use crate::{Pixels, Point};

use std::ops::Range;

/// The input method strategy of a widget.
#[derive(Debug, Clone, PartialEq)]
pub enum InputMethod<T = String> {
    /// Input method is disabled.
    Disabled,
    /// Input method is enabled.
    Enabled {
        /// The position at which the input method dialog should be placed.
        position: Point,
        /// The [`Purpose`] of the input method.
        purpose: Purpose,
        /// The preedit to overlay on top of the input method dialog, if needed.
        ///
        /// Ideally, your widget will show pre-edits on-the-spot; but, since that can
        /// be tricky, you can instead provide the current pre-edit here and the
        /// runtime will display it as an overlay (i.e. "Over-the-spot IME").
        preedit: Option<Preedit<T>>,
    },
}

/// The pre-edit of an [`InputMethod`].
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Preedit<T = String> {
    /// The current content.
    pub content: T,
    /// The selected range of the content.
    pub selection: Option<Range<usize>>,
    /// The text size of the content.
    pub text_size: Option<Pixels>,
}

impl<T> Preedit<T> {
    /// Creates a new empty [`Preedit`].
    pub fn new() -> Self
    where
        T: Default,
    {
        Self::default()
    }

    /// Turns a [`Preedit`] into its owned version.
    pub fn to_owned(&self) -> Preedit
    where
        T: AsRef<str>,
    {
        Preedit {
            content: self.content.as_ref().to_owned(),
            selection: self.selection.clone(),
            text_size: self.text_size,
        }
    }
}

impl Preedit {
    /// Borrows the contents of a [`Preedit`].
    pub fn as_ref(&self) -> Preedit<&str> {
        Preedit {
            content: &self.content,
            selection: self.selection.clone(),
            text_size: self.text_size,
        }
    }
}

/// The purpose of an [`InputMethod`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Purpose {
    /// No special hints for the IME (default).
    #[default]
    Normal,
    /// The IME is used for secure input (e.g. passwords).
    Secure,
    /// The IME is used to input into a terminal.
    ///
    /// For example, that could alter OSK on Wayland to show extra buttons.
    Terminal,
}

impl InputMethod {
    /// Merges two [`InputMethod`] strategies, prioritizing the first one when both open:
    /// ```
    /// # use iced_core::input_method::{InputMethod, Purpose, Preedit};
    /// # use iced_core::Point;
    ///
    /// let open = InputMethod::Enabled {
    ///     position: Point::ORIGIN,
    ///     purpose: Purpose::Normal,
    ///     preedit: Some(Preedit { content: "1".to_owned(), selection: None, text_size: None }),
    /// };
    ///
    /// let open_2 = InputMethod::Enabled {
    ///     position: Point::ORIGIN,
    ///     purpose: Purpose::Secure,
    ///     preedit: Some(Preedit { content: "2".to_owned(), selection: None, text_size: None }),
    /// };
    ///
    /// let mut ime = InputMethod::Disabled;
    ///
    /// ime.merge(&open);
    /// assert_eq!(ime, open);
    ///
    /// ime.merge(&open_2);
    /// assert_eq!(ime, open);
    /// ```
    pub fn merge<T: AsRef<str>>(&mut self, other: &InputMethod<T>) {
        if let InputMethod::Enabled { .. } = self {
            return;
        }

        *self = other.to_owned();
    }

    /// Returns true if the [`InputMethod`] is open.
    pub fn is_enabled(&self) -> bool {
        matches!(self, Self::Enabled { .. })
    }
}

impl<T> InputMethod<T> {
    /// Turns an [`InputMethod`] into its owned version.
    pub fn to_owned(&self) -> InputMethod
    where
        T: AsRef<str>,
    {
        match self {
            Self::Disabled => InputMethod::Disabled,
            Self::Enabled {
                position,
                purpose,
                preedit,
            } => InputMethod::Enabled {
                position: *position,
                purpose: *purpose,
                preedit: preedit.as_ref().map(Preedit::to_owned),
            },
        }
    }
}

/// Describes [input method](https://en.wikipedia.org/wiki/Input_method) events.
///
/// This is also called a "composition event".
///
/// Most keypresses using a latin-like keyboard layout simply generate a
/// [`keyboard::Event::KeyPressed`](crate::keyboard::Event::KeyPressed).
/// However, one couldn't possibly have a key for every single
/// unicode character that the user might want to type. The solution operating systems employ is
/// to allow the user to type these using _a sequence of keypresses_ instead.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Event {
    /// Notifies when the IME was opened.
    ///
    /// After getting this event you could receive [`Preedit`][Self::Preedit] and
    /// [`Commit`][Self::Commit] events. You should also start performing IME related requests
    /// like [`Shell::request_input_method`].
    ///
    /// [`Shell::request_input_method`]: crate::Shell::request_input_method
    Opened,

    /// Notifies when a new composing text should be set at the cursor position.
    ///
    /// The value represents a pair of the preedit string and the cursor begin position and end
    /// position. When it's `None`, the cursor should be hidden. When `String` is an empty string
    /// this indicates that preedit was cleared.
    ///
    /// The cursor range is byte-wise indexed.
    Preedit(String, Option<Range<usize>>),

    /// Notifies when text should be inserted into the editor widget.
    ///
    /// Right before this event, an empty [`Self::Preedit`] event will be issued.
    Commit(String),

    /// Notifies when the IME was disabled.
    ///
    /// After receiving this event you won't get any more [`Preedit`][Self::Preedit] or
    /// [`Commit`][Self::Commit] events until the next [`Opened`][Self::Opened] event. You should
    /// also stop issuing IME related requests like [`Shell::request_input_method`] and clear
    /// pending preedit text.
    ///
    /// [`Shell::request_input_method`]: crate::Shell::request_input_method
    Closed,
}
