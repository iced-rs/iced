//! Display fields that can be filled with text.
//!
//! A [`TextInput`] has some local [`State`].
//!
//! [`TextInput`]: struct.TextInput.html
//! [`State`]: struct.State.html
use crate::{bumpalo, style, Bus, Element, Length, Style, Widget};
use std::rc::Rc;

/// A field that can be filled with text.
///
/// # Example
/// ```
/// # use iced_web::{text_input, TextInput};
/// #
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
/// );
/// ```
#[allow(missing_debug_implementations)]
pub struct TextInput<'a, Message> {
    _state: &'a mut State,
    placeholder: String,
    value: String,
    is_secure: bool,
    width: Length,
    max_width: Length,
    padding: u16,
    size: Option<u16>,
    on_change: Rc<Box<dyn Fn(String) -> Message>>,
    on_submit: Option<Message>,
}

impl<'a, Message> TextInput<'a, Message> {
    /// Creates a new [`TextInput`].
    ///
    /// It expects:
    /// - some [`State`]
    /// - a placeholder
    /// - the current value
    /// - a function that produces a message when the [`TextInput`] changes
    ///
    /// [`TextInput`]: struct.TextInput.html
    /// [`State`]: struct.State.html
    pub fn new<F>(
        state: &'a mut State,
        placeholder: &str,
        value: &str,
        on_change: F,
    ) -> Self
    where
        F: 'static + Fn(String) -> Message,
    {
        Self {
            _state: state,
            placeholder: String::from(placeholder),
            value: String::from(value),
            is_secure: false,
            width: Length::Fill,
            max_width: Length::Shrink,
            padding: 0,
            size: None,
            on_change: Rc::new(Box::new(on_change)),
            on_submit: None,
        }
    }

    /// Converts the [`TextInput`] into a secure password input.
    ///
    /// [`TextInput`]: struct.TextInput.html
    pub fn password(mut self) -> Self {
        self.is_secure = true;
        self
    }
    
    /// Sets the width of the [`TextInput`].
    ///
    /// [`TextInput`]: struct.TextInput.html
    pub fn width(mut self, width: Length) -> Self {
        self.width = width;
        self
    }

    /// Sets the maximum width of the [`TextInput`].
    ///
    /// [`TextInput`]: struct.TextInput.html
    pub fn max_width(mut self, max_width: Length) -> Self {
        self.max_width = max_width;
        self
    }

    /// Sets the padding of the [`TextInput`].
    ///
    /// [`TextInput`]: struct.TextInput.html
    pub fn padding(mut self, units: u16) -> Self {
        self.padding = units;
        self
    }

    /// Sets the text size of the [`TextInput`].
    ///
    /// [`TextInput`]: struct.TextInput.html
    pub fn size(mut self, size: u16) -> Self {
        self.size = Some(size);
        self
    }

    /// Sets the message that should be produced when the [`TextInput`] is
    /// focused and the enter key is pressed.
    ///
    /// [`TextInput`]: struct.TextInput.html
    pub fn on_submit(mut self, message: Message) -> Self {
        self.on_submit = Some(message);
        self
    }
}

impl<'a, Message> Widget<Message> for TextInput<'a, Message>
where
    Message: 'static + Clone,
{
    fn node<'b>(
        &self,
        bump: &'b bumpalo::Bump,
        bus: &Bus<Message>,
        style_sheet: &mut style::Sheet<'b>,
    ) -> dodrio::Node<'b> {
        use dodrio::builder::*;
        use wasm_bindgen::JsCast;

        let width = style::length(self.width);
        let max_width = style::length(self.max_width);
        let padding_class =
            style_sheet.insert(bump, Style::Padding(self.padding));

        let on_change = self.on_change.clone();
        let event_bus = bus.clone();

        input(bump)
            .attr(
                "class",
                bumpalo::format!(in bump, "{}", padding_class).into_bump_str(),
            )
            .attr(
                "style",
                bumpalo::format!(
                    in bump,
                    "width: {}; max-width: {}; font-size: {}px",
                    width,
                    max_width,
                    self.size.unwrap_or(20)
                )
                .into_bump_str(),
            )
            .attr(
                "placeholder",
                bumpalo::format!(in bump, "{}", self.placeholder)
                    .into_bump_str(),
            )
            .attr(
                "value",
                bumpalo::format!(in bump, "{}", self.value).into_bump_str(),
            )
            .attr(
                "type",
                bumpalo::format!(in bump, "{}", if self.is_secure { "password" } else { "text" }).into_bump_str(),
            )
            .on("input", move |root, vdom, event| {
                let text_input = match event.target().and_then(|t| {
                    t.dyn_into::<web_sys::HtmlInputElement>().ok()
                }) {
                    None => return,
                    Some(text_input) => text_input,
                };

                event_bus.publish(on_change(text_input.value()), root);
                vdom.schedule_render();
            })
            .finish()
    }
}

impl<'a, Message> From<TextInput<'a, Message>> for Element<'a, Message>
where
    Message: 'static + Clone,
{
    fn from(text_input: TextInput<'a, Message>) -> Element<'a, Message> {
        Element::new(text_input)
    }
}

/// The state of a [`TextInput`].
///
/// [`TextInput`]: struct.TextInput.html
#[derive(Debug, Clone, Copy, Default)]
pub struct State;

impl State {
    /// Creates a new [`State`], representing an unfocused [`TextInput`].
    ///
    /// [`State`]: struct.State.html
    pub fn new() -> Self {
        Self::default()
    }
}
